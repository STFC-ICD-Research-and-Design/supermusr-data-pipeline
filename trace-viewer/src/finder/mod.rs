mod search_engine;
mod searcher;
mod task;

use chrono::TimeDelta;
use strum::{Display, EnumIter, EnumString};
use supermusr_common::{Channel, DigitizerId};

use crate::{Timestamp, messages::Cache};

pub(crate) use search_engine::SearchEngine;

#[derive(Default, Clone, EnumString, Display, EnumIter, Copy)]
pub(crate) enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
    //#[strum(to_string = "Capture in Realtime")]
    //Capture,
    //#[strum(to_string = "From End")]
    //End,
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
    Text(String),
    TraceSearchInProgress(f64),
    TraceSearchFinished,
    EventListSearchInProgress(f64),
    EventListSearchFinished,
    Successful{ num: usize, time: TimeDelta },
}

pub(crate) struct BrokerTopicInfo {
    pub(crate) offsets: (i64,i64),
    pub(crate) timestamps: (Timestamp,Timestamp),
}

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
    //Capture,
    //End,
}

#[derive(Clone)]
pub(crate) enum SearchTargetBy {
    ByChannels { channels: Vec<Channel> },
    ByDigitiserIds { digitiser_ids: Vec<DigitizerId> },
}

pub(crate) trait MessageFinder {
    type SearchMode;

    fn init_search(&mut self, target: SearchTarget) -> bool;

    fn status(&mut self) -> Option<SearchStatus>;

    fn results(&mut self) -> Option<SearchResults>;

    /// Takes ownership of the object's [BrokerInfo] struct if one exists.
    /// 
    /// This function does nothing if the object does not currently own the
    /// [StreamConsumer] struct, i.e. whilst a search is in progress.
    fn broker_info(&mut self) -> Option<Option<BrokerInfo>>;

    /// Polls the broker for topic info.
    fn init_poll_broker_info(&mut self) -> bool;

    async fn async_update(&mut self);
}
