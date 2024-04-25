mod metrics;
mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use kagiyama::{AlwaysReady, Watcher};
use parameters::{DetectorSettings, Mode, Polarity};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    conditional_init_tracer,
    tracer::{FutureRecordTracerExt, OptionalHeaderTracerExt, OtelTracer},
    Intensity,
};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
    flatbuffers::FlatBufferBuilder,
};
use tracing::{debug, error, metadata::LevelFilter, trace, trace_span, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long)]
    event_topic: String,

    /// Determines whether events should register as positive or negative intensity
    #[clap(long)]
    polarity: Polarity,

    /// Value of the intensity baseline
    #[clap(long, default_value = "0")]
    baseline: Intensity,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, the trace and event lists are saved here
    #[clap(long)]
    save_file: Option<PathBuf>,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let tracer = conditional_init_tracer!(args.otel_endpoint.as_deref(), LevelFilter::TRACE);

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&watcher);
    watcher.start_server(args.observability_address).await;

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .create()
        .expect("Kafka Producer should be created");

    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Kafka Consumer should be created");

    consumer
        .subscribe(&[&args.trace_topic])
        .expect("Kafka Consumer should subscribe to trace-topic");

    loop {
        match consumer.recv().await {
            Ok(m) => {
                let span = trace_span!("Trace Source Message");
                m.headers()
                    .conditional_extract_to_span(tracer.is_some(), &span);
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
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Trace,
                            ))
                            .inc();
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
                                    .conditional_inject_current_span_into_headers(tracer.is_some())
                                    .key("Digitiser Events List");

                                let future =
                                    producer.send_result(future_record).expect("Producer sends");

                                match future.await {
                                    Ok(_) => {
                                        trace!("Published event message");
                                        metrics::MESSAGES_PROCESSED.inc();
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                        metrics::FAILURES
                                            .get_or_create(&metrics::FailureLabels::new(
                                                metrics::FailureKind::KafkaPublishFailed,
                                            ))
                                            .inc();
                                    }
                                }
                                fbb.reset();
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                                metrics::FAILURES
                                    .get_or_create(&metrics::FailureLabels::new(
                                        metrics::FailureKind::UnableToDecodeMessage,
                                    ))
                                    .inc();
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", m.topic());
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Unknown,
                            ))
                            .inc();
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        }
    }
}
