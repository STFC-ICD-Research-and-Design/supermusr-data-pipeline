use chrono::{DateTime, Utc};
use clap::Parser;
use metrics::{counter, gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use supermusr_common::{CommonKafkaOpts, DigitizerId};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
    DigitizerAnalogTraceMessage,
};
use tokio::{task, time::sleep};
use tracing::{debug, error, trace, warn};

type MessageCounts = Arc<Mutex<HashMap<DigitizerId, usize>>>;

const METRIC_MSG_COUNT: &str = "digitiser_message_received_count";
const METRIC_MSG_RATE: &str = "digitiser_message_received_rate";
const METRIC_LAST_MSG_TIMESTAMP: &str = "digitiser_last_message_timestamp";
const METRIC_LAST_MSG_FRAME_NUMBER: &str = "digitiser_last_message_frame_number";
const METRIC_CHANNEL_COUNT: &str = "digitiser_channel_count";
const METRIC_SAMPLE_COUNT: &str = "digitiser_sample_count";

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// The Kafka topic that digitiser trace messages are consumed from
    #[clap(long)]
    trace_topic: String,

    #[clap(long, env, default_value = "127.0.0.1:9091")]
    metrics_address: SocketAddr,

    /// The interval at which the message rate is calculated in seconds
    #[clap(long, default_value_t = 5)]
    message_rate_interval: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &[args.trace_topic.as_str()],
    );

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

    metrics::describe_counter!(
        METRIC_MSG_RATE,
        metrics::Unit::CountPerSecond,
        "Rate of messages received"
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

    let recent_msg_counts: MessageCounts = Arc::new(Mutex::new(HashMap::new()));

    task::spawn(update_message_rate(
        recent_msg_counts.clone(),
        args.message_rate_interval,
    ));

    poll_kafka_msg(consumer, recent_msg_counts).await;

    Ok(())
}

fn get_digitiser_label(digitiser_id: DigitizerId) -> (String, String) {
    ("digitiser_id".to_string(), format!("{}", digitiser_id))
}

async fn update_message_rate(recent_msg_counts: MessageCounts, message_rate_interval: u64) {
    loop {
        // Wait a set period of time before calculating average.
        sleep(Duration::from_secs(message_rate_interval)).await;

        let mut recent_msg_counts = recent_msg_counts
            .lock()
            .expect("acquiring recent data mutex lock should not fail under normal conditions");
        // Calculate and record message rate for each digitiser.
        recent_msg_counts
            .iter_mut()
            .for_each(|(digitiser_id, recent_msg_count)| {
                // Calculate message rate for each digitiser and register metric.
                let msg_rate = *recent_msg_count as f64 / message_rate_interval as f64;
                let labels = [get_digitiser_label(*digitiser_id)];
                gauge!("digitiser_message_received_rate", &labels).set(msg_rate);

                // Reset recent message count for each digitiser.
                *recent_msg_count = 0;
            });
    }
}

fn process_message(data: &DigitizerAnalogTraceMessage<'_>, recent_msg_counts: MessageCounts) {
    let id = data.digitizer_id();
    let labels = [get_digitiser_label(id)];

    // Increment recent message count for the digitiser
    recent_msg_counts
        .lock()
        .expect("acquiring recent data mutex lock should not fail under normal conditions")
        .entry(id)
        .and_modify(|d| *d += 1)
        .or_insert(1);

    /* Metrics */
    // Message recieved count
    counter!("digitiser_message_received_count", &labels).increment(1);

    // Last message frame number
    let frame_number = data.metadata().frame_number();
    gauge!("digitiser_last_message_frame_number", &labels).set(frame_number as f64);

    // Channel count
    let channel_count = match data.channels() {
        Some(c) => c.len(),
        None => 0,
    };
    gauge!("digitiser_channel_count", &labels).set(channel_count as f64);

    // Sample count
    if let Some(c) = data.channels() {
        for channel_index in 0..c.len() {
            let num_samples = match c.get(channel_index).voltage() {
                Some(v) => v.len(),
                None => 0,
            };
            let channel_labels = [("channel_index".to_string(), format!("{}", channel_index))];

            gauge!(
                "digitiser_sample_count",
                &[&labels[..], &channel_labels[..]].concat()
            )
            .set(num_samples as f64);
        }
    }

    // Last message timestamp
    let timestamp: DateTime<Utc> = data
        .metadata()
        .timestamp()
        .copied()
        .expect("timestamp should be present")
        .try_into()
        .expect("timestamp should be valid");

    gauge!("digitiser_last_message_timestamp", &labels).set(
        timestamp
            .timestamp_nanos_opt()
            .expect("timestamp should be representable in nanoseconds") as f64,
    );

    debug!(
        "Trace packet: dig. ID: {}, metadata: {:?}",
        data.digitizer_id(),
        data.metadata()
    );
}

/// Poll kafka messages and update digitiser data.
async fn poll_kafka_msg(consumer: StreamConsumer, recent_msg_counts: MessageCounts) {
    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                trace!(
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
                            Ok(data) => process_message(&data, Arc::clone(&recent_msg_counts)),
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }

                if let Err(e) = consumer.commit_message(&msg, CommitMode::Async) {
                    error!("Failed to commit message consume: {e}");
                }
            }
        };
    }
}
