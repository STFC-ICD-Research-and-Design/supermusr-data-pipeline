mod file;
mod metrics;

use anyhow::Result;
use clap::Parser;
use kagiyama::{AlwaysReady, Watcher};
use rdkafka::{
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: String,

    #[clap(long)]
    password: String,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long, default_value = ".")]
    output: PathBuf,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&mut watcher);
    watcher.start_server(args.observability_address).await;

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &args.broker)
        .set("security.protocol", "sasl_plaintext")
        .set("sasl.mechanisms", "SCRAM-SHA-256")
        .set("sasl.username", &args.username)
        .set("sasl.password", &args.password)
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&args.trace_topic])?;

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                log::info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                                metrics::MESSAGES_RECEIVED
                                    .get_or_create(&metrics::MessagesReceivedLabels::new(
                                        metrics::MessageKind::Trace,
                                    ))
                                    .inc();
                                if let Err(e) = file::create(&args.output, data) {
                                    log::warn!("Failed to save file: {}", e);
                                    metrics::FAILURES
                                        .get_or_create(&metrics::FailureLabels::new(
                                            metrics::FailureKind::FileWriteFailed,
                                        ))
                                        .inc();
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                                metrics::FAILURES
                                    .get_or_create(&metrics::FailureLabels::new(
                                        metrics::FailureKind::UnableToDecodeMessage,
                                    ))
                                    .inc();
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
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
