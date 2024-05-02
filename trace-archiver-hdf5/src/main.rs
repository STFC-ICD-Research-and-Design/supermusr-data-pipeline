mod file;
mod metrics;

use crate::file::TraceFile;
use anyhow::Result;
use clap::Parser;
use kagiyama::{prometheus::metrics::info::Info, AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tracing::{debug, info, warn};

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
    file: PathBuf,

    #[clap(long)]
    digitizer_count: usize,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    debug!("Args: {:?}", args);

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&mut watcher);
    {
        let output_files = Info::new(vec![("trace".to_string(), args.file.display().to_string())]);

        let mut registry = watcher.metrics_registry();
        registry.register("output_files", "Configured output filenames", output_files);
    }
    watcher.start_server(args.observability_address).await;

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    consumer.subscribe(&[&args.trace_topic])?;

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
                                metrics::MESSAGES_RECEIVED
                                    .get_or_create(&metrics::MessagesReceivedLabels::new(
                                        metrics::MessageKind::Trace,
                                    ))
                                    .inc();
                                if let Err(e) = trace_file.push(&data) {
                                    warn!("Failed to save traces to file: {}", e);
                                    metrics::FAILURES
                                        .get_or_create(&metrics::FailureLabels::new(
                                            metrics::FailureKind::FileWriteFailed,
                                        ))
                                        .inc();
                                }
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
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Unknown,
                            ))
                            .inc();
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}
