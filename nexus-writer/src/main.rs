mod nexus;
mod metrics;

use nexus::Nexus;
use anyhow::{anyhow, Result};
use chrono as _;
use ndarray as _;
use ndarray_stats as _;
use clap::Parser;
use kagiyama::{prometheus::metrics::info::Info, AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_streaming_types::{
    aev1_frame_assembled_event_v1_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    dat1_digitizer_analog_trace_v1_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
};

//  To run trace-reader
// cargo run -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 18 --random-sample

// To run nexus-writer
// cargo run -- --broker localhost:19092 --consumer-group nexus-writer --trace-topic Traces --file-name output.nx
#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long)]
    consumer_group: String,

    #[clap(long)]
    event_topic: Option<String>,

    #[clap(long)]
    trace_topic: Option<String>,

    #[clap(long)]
    histogram_topic: Option<String>,

    #[clap(long)]
    file_name: PathBuf,

    #[clap(long)]
    digitizer_count: Option<usize>,

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
    {/*
        let output_files = Info::new(vec![
            (
                "event".to_string(),
                match args.event_file {
                    Some(ref f) => f.display().to_string(),
                    None => "none".into(),
                },
            ),
            (
                "trace".to_string(),
                match args.trace_file {
                    Some(ref f) => f.display().to_string(),
                    None => "none".into(),
                },
            ),
        ]);

        let mut registry = watcher.metrics_registry();
        registry.register("output_files", "Configured output filenames", output_files);
         */
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

    let topics_to_subscribe: Vec<&str> = vec![args.event_topic.as_deref(), args.trace_topic.as_deref(), args.histogram_topic.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if topics_to_subscribe.is_empty() {
        return Err(anyhow!(
            "Nothing to do (no message type requested to be saved)"
        ));
    }
    consumer.subscribe(&topics_to_subscribe)?;
    let mut nexus = Nexus::new();

    let mut count = 0;

    loop {
        if count == 0 {
            nexus.create_file(&args.file_name)?;
        } else if count == 4 {
            nexus.close_file()?;
        }
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
                    if nexus.is_running() {
                        if args.trace_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
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
                                    if let Err(e) = nexus.add_trace_group(&data) {
                                        log::warn!("Failed to save traces to file: {}", e);
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
                        } else if args.event_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            // todo
                        } else if args.histogram_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            // todo
                        } else {
                            // todo
                        }
                    } else {
                        if args.trace_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            // todo
                        } else {
                            log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                            metrics::MESSAGES_RECEIVED
                                .get_or_create(&metrics::MessagesReceivedLabels::new(
                                    metrics::MessageKind::Unknown,
                                ))
                                .inc();
                        }
                    }
                }
                consumer.commit_message(&msg, CommitMode::Async).unwrap();
                count += 1;
            }
        };
    }
}
