mod file;

use crate::file::{EventFile, TraceFile};
use anyhow::{anyhow, Result};
use clap::Parser;
use metrics::counter;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::metrics::{
    failures::{self, FailureKind},
    messages_received::{self, MessageKind},
    metric_names::{FAILURES, MESSAGES_RECEIVED},
};
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
    event_topic: Option<String>,

    #[clap(long)]
    event_file: Option<PathBuf>,

    #[clap(long)]
    trace_topic: Option<String>,

    #[clap(long)]
    trace_file: Option<PathBuf>,

    #[clap(long)]
    digitizer_count: Option<usize>,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    debug!("Args: {:?}", args);

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

    let topics_to_subscribe: Vec<String> = vec![args.event_topic, args.trace_topic]
        .into_iter()
        .flatten()
        .collect();
    let topics_to_subscribe: Vec<&str> = topics_to_subscribe.iter().map(|i| i.as_ref()).collect();
    if topics_to_subscribe.is_empty() {
        return Err(anyhow!(
            "Nothing to do (no message type requested to be saved)"
        ));
    }
    consumer.subscribe(&topics_to_subscribe)?;

    // Metrics
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

    let mut event_file = match args.event_file {
        Some(filename) => Some(EventFile::create(&filename)?),
        None => None,
    };

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
                    if event_file.is_some()
                        && frame_assembled_event_list_message_buffer_has_identifier(payload)
                    {
                        match root_as_frame_assembled_event_list_message(payload) {
                            Ok(data) => {
                                info!("Event packet: metadata: {:?}", data.metadata());
                                counter!(
                                    MESSAGES_RECEIVED,
                                    &[messages_received::get_label(MessageKind::Event)]
                                )
                                .increment(1);
                                if let Err(e) = event_file.as_mut().unwrap().push(&data) {
                                    warn!("Failed to save events to file: {}", e);
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
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    } else if trace_file.is_some()
                        && digitizer_analog_trace_message_buffer_has_identifier(payload)
                    {
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
                                if let Err(e) = trace_file.as_mut().unwrap().push(&data) {
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
                            &[messages_received::get_label(MessageKind::Unknown)]
                        )
                        .increment(1);
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}
