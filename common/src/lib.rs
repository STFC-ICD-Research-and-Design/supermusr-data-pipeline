pub mod metrics;
pub mod spanned;
pub mod tracer;
mod version;

use clap::Args;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
};

pub type DigitizerId = u8;
pub type Time = u32;
pub type Channel = u32;
pub type Intensity = u16;

pub type FrameNumber = u32;
pub type SampleRate = u64;

pub const CHANNELS_PER_DIGITIZER: usize = 8;

pub fn channel_index(digitizer_index: usize, channel_index: usize) -> usize {
    (digitizer_index * CHANNELS_PER_DIGITIZER) + channel_index
}

#[derive(Default)]
pub struct EventData {
    pub time: Vec<Time>,
    pub channel: Vec<Channel>,
    pub voltage: Vec<Intensity>,
}

#[derive(Clone, Debug, Args)]
pub struct CommonKafkaOpts {
    /// Address of Kafka message broker, should have format `host:port`
    #[clap(long)]
    pub broker: String,

    /// Optional Kafka username.
    /// If provided, a corresponding password is required.
    #[clap(long)]
    pub username: Option<String>,

    /// Optional Kafka password.
    /// If provided, a corresponding username is requred.
    #[clap(long)]
    pub password: Option<String>,
}

pub fn generate_kafka_client_config(
    broker_address: &String,
    username: &Option<String>,
    password: &Option<String>,
) -> ClientConfig {
    let mut client_config = ClientConfig::new()
        .set("bootstrap.servers", broker_address)
        .set("compression.type", "zstd")
        .clone();

    // Allow for authenticated Kafka connection if details are provided
    if let (Some(sasl_username), Some(sasl_password)) = (username, password) {
        client_config
            .set("sasl.mechanisms", "SCRAM-SHA-256")
            .set("security.protocol", "sasl_plaintext")
            .set("sasl.username", sasl_username)
            .set("sasl.password", sasl_password);
    }
    client_config
}

pub fn create_default_consumer(
    broker_address: &String,
    username: &Option<String>,
    password: &Option<String>,
    consumer_group: &String,
    topics_to_subscribe: Option<&[&str]>,
) -> Result<StreamConsumer, KafkaError> {
    // Setup consumer with arguments and default parameters.
    let consumer: StreamConsumer = generate_kafka_client_config(broker_address, username, password)
        .set("group.id", consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    // Subscribe to if topics are provided.
    if let Some(topics_to_subscribe) = topics_to_subscribe {
        // Note this fails if the topics list is empty
        consumer.subscribe(topics_to_subscribe)?;
    }

    Ok(consumer)
}
