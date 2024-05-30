mod file;
mod metrics;

use crate::file::TraceFile;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use kagiyama::{prometheus::metrics::info::Info, AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    Timestamp,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage,
    },
    ecs_6s4t_run_stop_generated::run_stop_buffer_has_identifier,
    ecs_pl72_run_start_generated::run_start_buffer_has_identifier,
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
    control_topic: Option<String>,

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

    let topics_to_subscribe = [
        args.control_topic.as_deref(),
        Some(args.trace_topic.as_str()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    consumer.subscribe(&topics_to_subscribe)?;

    let mut file = match args.control_topic {
        // If a control topic is provided, a file will be created for each run.
        Some(_) => None,
        // If a a control topic is not provided, a persistent file is used instead.
        None => Some(TraceFile::create(&args.file, args.digitizer_count)?),
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
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        // A message has been received from the trace topic.
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => process_trace_topic_data(&data, &mut file),
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                                metrics::FAILURES
                                    .get_or_create(&metrics::FailureLabels::new(
                                        metrics::FailureKind::UnableToDecodeMessage,
                                    ))
                                    .inc();
                            }
                        }
                    } else if args.control_topic == Some(msg.topic().to_string()) {
                        // A message has been received from the control topic.
                        if run_start_buffer_has_identifier(payload) {
                            debug!("New run start.");
                            // Start recording trace data to file.
                            if file.is_none() {
                                let filename = generate_filename(msg.timestamp());
                                file = Some(TraceFile::create(&filename, args.digitizer_count)?);
                            }
                            // If file already exists, do nothing.
                        } else if run_stop_buffer_has_identifier(payload) {
                            debug!("New run stop.");
                            // Stop recording trace data to file.
                            file = None;
                        } else {
                            warn!("Incorrect message identifier on topic \"{}\"", msg.topic());
                        }
                    } else {
                        // The message kind is unknown.
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

fn generate_filename(timestamp: Timestamp) -> PathBuf {
    //  TODO: Check this unwrap does not cause any issues.
    let timestamp =
        DateTime::<Utc>::from_timestamp_millis(timestamp.to_millis().unwrap_or_default())
            .unwrap_or_default();
    PathBuf::from(format!("{:?}.hdf5", timestamp))
}

fn process_trace_topic_data(data: &DigitizerAnalogTraceMessage<'_>, file: &mut Option<TraceFile>) {
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
    if let Some(file) = file {
        info!("Writing trace data to \"{}\"", file.filename());
        if let Err(e) = file.push(data) {
            warn!("Failed to save traces to file: {}", e);
            metrics::FAILURES
                .get_or_create(&metrics::FailureLabels::new(
                    metrics::FailureKind::FileWriteFailed,
                ))
                .inc();
        }
    }
}
