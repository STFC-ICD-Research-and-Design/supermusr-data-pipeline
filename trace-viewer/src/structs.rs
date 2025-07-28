use cfg_if::cfg_if;
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
use crate::{messages::VectorisedCache, Channel, DigitizerId, Timestamp};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use clap::Args;
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Topics {
    /// Kafka trace topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub trace_topic: String,

    /// Kafka digitiser event list topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub digitiser_event_topic: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Select {
    /// The timestamp of the frame to search for, should be in the format "YYYY-MM-DD hh:mm:ss.f <timezone>".
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) timestamp: Option<Timestamp>,

    /// The digitiser Ids to search for.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) digitiser_ids: Option<Vec<DigitizerId>>,

    /// The channels to search for.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) channels: Option<Vec<Channel>>,

    /// The maximum number of messages to collect.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) number: Option<usize>,
}


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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub cache: VectorisedCache
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
