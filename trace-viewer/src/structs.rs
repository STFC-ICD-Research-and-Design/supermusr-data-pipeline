use cfg_if::cfg_if;
use chrono::TimeDelta;
use strum::{Display, EnumIter, EnumString};
use crate::{messages::Cache, Channel, DigitizerId, Timestamp};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use clap::Args;
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Topics {
    /// Kafka trace topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) trace_topic: String,

    /// Kafka digitiser event list topic.
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) digitiser_event_topic: String,
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "ssr", derive(Args))]
pub struct Select {
    /// The timestamp of the frame to search for, should be in the format "YYYY-MM-DD hh:mm:ss.f <timezone>".
    #[cfg_attr(feature = "ssr", clap(long))]
    pub(crate) timestamp: Option<Timestamp>,
}


#[derive(Default, Clone, EnumString, Display, EnumIter, Copy)]
pub(crate) enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
}

#[derive(Default, Clone, EnumString, Display, EnumIter, Copy)]
pub(crate) enum SearchBy {
    #[default]
    #[strum(to_string = "By Channels")]
    ByChannels,
    #[strum(to_string = "By Digitiser Ids")]
    ByDigitiserIds,
}

#[derive(Default)]
pub(crate) enum SearchStatus {
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

#[derive(Clone)]
pub(crate) struct BrokerTopicInfo {
    pub(crate) offsets: (i64, i64),
    pub(crate) timestamps: (Timestamp, Timestamp),
}

#[derive(Clone)]
pub(crate) struct BrokerInfo {
    pub(crate) trace: BrokerTopicInfo,
    pub(crate) events: BrokerTopicInfo,
}

#[derive(Default)]
pub(crate) struct SearchResults {
    pub(crate) cache: Cache,
}

#[derive(Clone)]
pub(crate) struct SearchTarget {
    pub(crate) mode: SearchTargetMode,
    pub(crate) by: SearchTargetBy,
    pub(crate) number: usize,
}

#[derive(Clone)]
pub(crate) enum SearchTargetMode {
    Timestamp { timestamp: Timestamp },
}

#[derive(Clone)]
pub(crate) enum SearchTargetBy {
    ByChannels { channels: Vec<Channel> },
    ByDigitiserIds { digitiser_ids: Vec<DigitizerId> },
}
