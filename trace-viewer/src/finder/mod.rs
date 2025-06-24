mod engine;
mod searcher;
mod task;

use chrono::Duration;
use strum::{Display, EnumIter, EnumString};
use supermusr_common::{Channel, DigitizerId};

use crate::{Timestamp, messages::Cache};

pub(crate) use engine::SearchEngine;

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
    TraceSearchInProgress(u32),
    TraceSearchFinished,
    EventListSearchInProgress(u32),
    EventListSearchFinished,
    Successful,
}

#[derive(Default)]
pub(crate) struct SearchResults {
    pub(crate) time: Duration,
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

    async fn update(&mut self);
}
