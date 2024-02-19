use std::net::SocketAddr;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use metrics::{counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

const METRIC_MSG_COUNT: &str = "digitiser_message_received_count";
const METRIC_LAST_MSG_TIMESTAMP: &str = "digitiser_last_message_timestamp";
const METRIC_LAST_MSG_FRAME_NUMBER: &str = "digitiser_last_message_frame_number";
const METRIC_CHANNEL_COUNT: &str = "digitiser_channel_count";
const METRIC_SAMPLE_COUNT: &str = "digitiser_sample_count";

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

    #[clap(long, default_value = "127.0.0.1:9090")]
    metrics_address: SocketAddr,

    #[clap(long, default_value_t = 5)]
    message_rate_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

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

    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(args.metrics_address)
        .install()
        .expect("prometheus metrics exporter should be setup");

    metrics::describe_counter!(
        METRIC_MSG_COUNT,
        metrics::Unit::Count,
        "Number of messages received"
    );

    metrics::describe_gauge!(
        METRIC_LAST_MSG_TIMESTAMP,
        metrics::Unit::Nanoseconds,
        "Timestamp of last message received (ns)"
    );

    metrics::describe_gauge!(
        METRIC_LAST_MSG_FRAME_NUMBER,
        metrics::Unit::Count,
        "Frame number of last message received"
    );

    metrics::describe_gauge!(
        METRIC_CHANNEL_COUNT,
        metrics::Unit::Count,
        "Number of channels in last message received"
    );

    metrics::describe_gauge!(
        METRIC_SAMPLE_COUNT,
        metrics::Unit::Count,
        "Number of samples in last message received"
    );

    poll_kafka_msg(consumer).await;

    Ok(())
}

/// Poll kafka messages and update digitiser data.
async fn poll_kafka_msg(consumer: StreamConsumer) {
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
                                let id = data.digitizer_id();
                                let labels = [("digitiser_id", format!("{}", id))];

                                /* Metrics */
                                // Message recieved count
                                counter!("digitiser_message_received_count", &labels).increment(1);

                                // Last message frame number
                                let frame_number = data.metadata().frame_number();
                                gauge!("digitiser_last_message_frame_number", &labels)
                                    .set(frame_number as f64);

                                // Channel count
                                let channel_count = match data.channels() {
                                    Some(c) => c.len(),
                                    None => 0,
                                };
                                gauge!("digitiser_channel_count", &labels)
                                    .set(channel_count as f64);

                                // Sample count
                                if let Some(c) = data.channels() {
                                    for channel_index in 0..c.len() {
                                        let num_samples =
                                            c.get(channel_index).voltage().unwrap().len();
                                        let channel_labels =
                                            [("channel_index", format!("{}", channel_index))];

                                        gauge!(
                                            "digitiser_sample_count",
                                            &[&labels[..], &channel_labels[..]].concat()
                                        )
                                        .set(num_samples as f64);
                                    }
                                }

                                // Last message timestamp
                                let timestamp: DateTime<Utc> =
                                    data.metadata().timestamp().copied().unwrap().into();
                                gauge!("digitiser_last_message_timestamp", &labels)
                                    .set(timestamp.timestamp_nanos_opt().unwrap() as f64);

                                log::info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}
