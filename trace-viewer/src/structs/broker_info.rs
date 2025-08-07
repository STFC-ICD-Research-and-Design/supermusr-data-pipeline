use crate::{Channel, DigitizerId, Timestamp, messages::VectorisedCache};
use cfg_if::cfg_if;
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrokerTopicInfo {
    pub offsets: (i64, i64),
    pub timestamps: Option<(Timestamp, Timestamp)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrokerInfo {
    pub timestamp: Timestamp,
    pub trace: BrokerTopicInfo,
    pub events: BrokerTopicInfo,
}
