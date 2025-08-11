use crate::{Channel, DigitizerId, Timestamp};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SearchStatus {
    #[default]
    Off,
    TraceSearchInProgress(f64),
    TraceSearchFinished,
    EventListSearchInProgress(f64),
    EventListSearchFinished,
    Successful {
        num: usize,
        time: TimeDelta,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchTarget {
    pub mode: SearchTargetMode,
    pub by: SearchTargetBy,
    pub number: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchTargetMode {
    Timestamp { timestamp: Timestamp },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchTargetBy {
    ByChannels { channels: Vec<Channel> },
    ByDigitiserIds { digitiser_ids: Vec<DigitizerId> },
}
