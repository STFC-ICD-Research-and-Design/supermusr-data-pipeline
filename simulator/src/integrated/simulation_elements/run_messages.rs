use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SendRunStart {
    pub(crate) name: String,
    pub(crate) instrument: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SendRunStop {
    pub(crate) name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SendRunLogData {
    pub(crate) source_name: String,
    pub(crate) value_type: String,
    pub(crate) value: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SendSampleEnvLog {
    pub(crate) name: String,
    pub(crate) channel: Option<i32>,
    pub(crate) time_delta: Option<f64>,
    pub(crate) values_type: String,
    pub(crate) message_counter: Option<i64>,
    pub(crate) location: String,
    pub(crate) values: Vec<String>,
    pub(crate) timestamps: Option<Vec<DateTime<Utc>>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SendAlarm {
    pub(crate) source_name: String,
    pub(crate) severity: String,
    pub(crate) message: String,
}
