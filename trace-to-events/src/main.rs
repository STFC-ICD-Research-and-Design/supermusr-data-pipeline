mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use parameters::{DetectorSettings, Mode, Polarity};
use rdkafka::{
    consumer::{CommitMode, Consumer},
    producer::{FutureProducer, FutureRecord},
    Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    init_tracer,
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    tracer::{FutureRecordTracerExt, OptionalHeaderTracerExt, TracerEngine, TracerOptions},
    CommonKafkaOpts, Intensity,
};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
    flatbuffers::FlatBufferBuilder,
};
use tokio::task::JoinSet;
use tracing::{debug, error, metadata::LevelFilter, trace, trace_span, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long)]
    consumer_group: String,

    /// The Kafka topic that trace messages are consumed from
    #[clap(long)]
    trace_topic: String,

    /// Topic to publish digitiser event messages to
    #[clap(long)]
    event_topic: String,

    /// Determines whether events should register as positive or negative intensity
    #[clap(long)]
    polarity: Polarity,

    /// Value of the intensity baseline
    #[clap(long, default_value = "0")]
    baseline: Intensity,

    /// Endpoint on which OpenMetrics flavour metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, the trace and event lists are saved here
    #[clap(long)]
    save_file: Option<PathBuf>,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// If OpenTelemetry is used then it uses the following tracing level
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_level
    ));

    let kafka_opts = args.common_kafka_options;

    let client_config = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    );

    let producer: FutureProducer = client_config.create()?;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &[args.trace_topic.as_str()],
    );

    // Install exporter and register metrics
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(args.observability_address)
        .install()?;

    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        MESSAGES_PROCESSED,
        metrics::Unit::Count,
        "Number of messages processed"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );

    let mut kafka_producer_thread_set = JoinSet::new();

    loop {
        match consumer.recv().await {
            Ok(m) => {
                let span = trace_span!("Trace Source Message");
                m.headers()
                    .conditional_extract_to_span(tracer.use_otel(), &span);
                let _guard = span.enter();

                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                if let Some(payload) = m.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Trace)]
                        )
                        .increment(1);
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(thing) => {
                                let mut fbb = FlatBufferBuilder::new();
                                processing::process(
                                    &mut fbb,
                                    &thing,
                                    &DetectorSettings {
                                        polarity: &args.polarity,
                                        baseline: args.baseline,
                                        mode: &args.mode,
                                    },
                                    args.save_file.as_deref(),
                                );

                                let future_record = FutureRecord::to(&args.event_topic)
                                    .payload(fbb.finished_data())
                                    .conditional_inject_current_span_into_headers(tracer.use_otel())
                                    .key("Digitiser Events List");

                                match producer.send_result(future_record) {
                                    Ok(future) => {
                                        kafka_producer_thread_set.spawn(async move {
                                            match future.await {
                                                Ok(_) => {
                                                    trace!("Published event message");
                                                    counter!(MESSAGES_PROCESSED).increment(1);
                                                }
                                                Err(e) => {
                                                    error!("{:?}", e);
                                                    counter!(
                                                        FAILURES,
                                                        &[failures::get_label(
                                                            FailureKind::KafkaPublishFailed
                                                        )]
                                                    )
                                                    .increment(1);
                                                }
                                            }
                                        });
                                    }
                                    Err(e) => error!("Failed to enqueue message: {:?}", e),
                                }

                                fbb.reset();
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                                counter!(
                                    FAILURES,
                                    &[failures::get_label(FailureKind::UnableToDecodeMessage)]
                                )
                                .increment(1);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", m.topic());
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Unexpected)]
                        )
                        .increment(1);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        }
    }
}
