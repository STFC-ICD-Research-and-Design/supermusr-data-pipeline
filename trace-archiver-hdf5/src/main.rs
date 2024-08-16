mod file;

use crate::file::TraceFile;
use clap::Parser;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_RECEIVED},
    },
    CommonKafkaOpts,
};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tracing::{debug, error, info, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// The Kafka topic that trace messages are consumed from
    #[clap(long)]
    trace_topic: String,

    /// HDF5 file to write to
    #[clap(long)]
    file: PathBuf,

    /// Number of digitisers.
    #[clap(long)]
    digitizer_count: usize,

    /// Endpoint on which OpenMetrics flavour metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    debug!("Args: {:?}", args);

    let kafka_opts = args.common_kafka_options;

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
        .install()
        .expect("Prometheus metrics exporter should be setup");

    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );

    let mut trace_file = TraceFile::create(&args.file, args.digitizer_count)?;

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        // In other words, if the message is from the trace topic.
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                                counter!(
                                    MESSAGES_RECEIVED,
                                    &[messages_received::get_label(MessageKind::Trace)]
                                )
                                .increment(1);
                                if let Err(e) = trace_file.push(&data) {
                                    warn!("Failed to save traces to file: {}", e);
                                    counter!(
                                        FAILURES,
                                        &[failures::get_label(FailureKind::FileWriteFailed)]
                                    )
                                    .increment(1);
                                }
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
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Unexpected)]
                        )
                        .increment(1);
                    }
                }

                if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                    error!("Failed to commit message consume: {e}");
                }
            }
        };
    }
}
