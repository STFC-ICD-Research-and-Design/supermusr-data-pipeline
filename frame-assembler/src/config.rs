use anyhow::Result;
use serde::Deserialize;
use std::{fs, time::Duration};

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub(crate) broker: BrokerConfig,
    pub(crate) topics: TopicConfig,
    processing: ProcessingConfig,
    pub(crate) digitisers: DigitiserConfigs,
}

impl Config {
    pub(crate) fn from_file(path: &str) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct BrokerConfig {
    pub address: String,
    pub username: String,
    pub password: String,
    pub group: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TopicConfig {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Deserialize)]
struct ProcessingConfig {
    #[serde(with = "duration_format", default = "duration_default")]
    frame_timeout: Duration,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DigitiserConfig {
    pub id: u8,
}

pub(crate) type DigitiserConfigs = Vec<DigitiserConfig>;

mod duration_format {
    use super::Duration;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> core::result::Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let t = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(t))
    }
}

fn duration_default() -> Duration {
    Duration::from_millis(500)
}
