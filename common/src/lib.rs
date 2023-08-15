pub mod metrics;

pub type DigitizerId = u8;
pub type Time = u32;
pub type Channel = u32;
pub type Intensity = u16;

pub type FrameNumber = u32;
pub type SampleRate = u64;

use rdkafka::config::ClientConfig;

#[derive(Default)]
pub struct EventData {
    pub time: Vec<Time>,
    pub channel: Vec<Channel>,
    pub voltage: Vec<Intensity>,
}

pub const CHANNELS_PER_DIGITIZER: usize = 8;

pub fn channel_index(digitizer_index: usize, channel_index: usize) -> usize {
    (digitizer_index * CHANNELS_PER_DIGITIZER) + channel_index
}

pub fn generate_client_config(broker_address: &String, opt_username: Option<&String>, opt_password: Option<&String>) -> ClientConfig {
    let mut client_config = ClientConfig::new()
        .set("bootstrap.servers", broker_address)
        .clone();

    // Allow for authenticated Kafka connection if details are provided
    if let (Some(username), Some(password)) = (opt_username, opt_password) {
        client_config
            .set("sasl.mechanisms", "SCRAM-SHA-256")
            .set("security.protocol", "sasl_plaintext")
            .set("sasl.username", username)
            .set("sasl.password", password);
    }
    return client_config;
}