use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use metrics::{counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
    frame_metadata_v1_generated::GpsTime,
};
use tokio::task;

/*
Metrics to be reported and labels they should carry:

    "digitiser_message_received_count" (dig. ID) :          Number of messages received
    "digitiser_last_message_timestamp" (dig. ID) :          Timestamp of last message received (in ns unix epoch)
    "digitiser_last_message_frame_number" (dig. ID) :       Frame number of last message received
    "digitiser_channel_count" (dig. ID):                    Number of channels in last message received
    "digitiser_sample_count" (dig. ID, channel number):     Number of samples in last message received
*/

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
    prometheus: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long, default_value_t = 5)]
    message_rate_interval: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

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
        .with_http_listener(
            std::net::SocketAddr::from_str(args.prometheus.as_str())
                .expect("Should be able to cast broker address to SocketAddr type."),
        )
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

    task::spawn(poll_kafka_msg(consumer));

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
                                let labels = [("dynamic_key", format!("{}", id))];

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
                                let num_samples_in_first_channel = match data.channels() {
                                    Some(c) => c.get(0).voltage().unwrap().len(),
                                    None => 0,
                                };
                                gauge!("digitiser_sample_count", &labels)
                                    .set(num_samples_in_first_channel as f64);

                                // Last message timestamp
                                let timestamp: Option<GpsTime> =
                                    data.metadata().timestamp().copied().map(|t| t.into());
                                gauge!("digitiser_last_message_timestamp", &labels)
                                    .set(timestamp.unwrap().nanosecond() as f64);

                                /* Logging */
                                log::info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );

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
