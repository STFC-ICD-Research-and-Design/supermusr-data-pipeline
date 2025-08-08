use crate::{
    Channel, DigitizerId, Timestamp, messages::VectorisedCache,
};
use cfg_if::cfg_if;
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Copy)]
pub enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
    #[strum(to_string = "At Random")]
    Random,
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Copy)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResults {
    Cancelled,
    Successful { cache: VectorisedCache },
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::server_functions::SessionError;
        
        impl SearchResults {
            pub fn cache(&self) -> Result<&VectorisedCache, SessionError> {
                match self {
                    SearchResults::Cancelled => Err(SessionError::SearchCancelled),
                    SearchResults::Successful { cache } => Ok(cache),
                }
            }
        }
    }
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
