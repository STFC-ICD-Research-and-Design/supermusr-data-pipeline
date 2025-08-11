use crate::{
    Channel, DigitizerId, Timestamp
};
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
    #[strum(to_string = "At Random")]
    Random,
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub enum SearchBy {
    #[default]
    #[strum(to_string = "By Channels")]
    ByChannels,
    #[strum(to_string = "By Digitiser Ids")]
    ByDigitiserIds,
}

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

#[derive(Clone, EnumIter, EnumString, Debug, Display, Serialize, Deserialize)]
pub enum SearchTargetMode {
    Timestamp { timestamp: Timestamp },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SearchTargetBy {
    ByChannels { channels: Vec<Channel> },
    ByDigitiserIds { digitiser_ids: Vec<DigitizerId> },
}
