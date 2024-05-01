mod file;
mod metrics;

use crate::file::TraceFile;
use anyhow::{anyhow, Result};
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
    trace_topic: Option<String>,

    #[clap(long)]
    trace_file: Option<PathBuf>,

    #[clap(long)]
    digitizer_count: Option<usize>,

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
        let output_files = Info::new(vec![(
            "trace".to_string(),
            match args.trace_file {
                Some(ref f) => f.display().to_string(),
                None => "none".into(),
            },
        )]);

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

    if let Some(trace_topic) = args.trace_topic {
        consumer.subscribe(&[&trace_topic])?;
    } else {
        return Err(anyhow!(
            "Nothing to do (no message type requested to be saved)"
        ));
    }

    let mut trace_file = match args.trace_file {
        Some(filename) => Some(TraceFile::create(
            &filename,
            args.digitizer_count
                .expect("digitizer count should be provided"),
        )?),
        None => None,
    };

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
                    if trace_file.is_some()
                        && digitizer_analog_trace_message_buffer_has_identifier(payload)
                    {
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
                                if let Err(e) = trace_file.as_mut().unwrap().push(&data) {
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
