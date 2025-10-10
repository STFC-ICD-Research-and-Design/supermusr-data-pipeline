use crate::{Channel, DigitizerId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchTarget {
    pub mode: SearchTargetMode,
    pub by: SearchTargetBy,
    pub number: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchTargetMode {
    Timestamp {
        timestamp: Timestamp,
    },
    Dragnet {
        timestamp: Timestamp,
        backstep: i64,
        forward_distance: usize,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchTargetBy {
    All,
    ByChannels { channels: Vec<Channel> },
    ByDigitiserIds { digitiser_ids: Vec<DigitizerId> },
}
