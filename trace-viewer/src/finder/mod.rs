mod engine;
mod searcher;
mod task;

use chrono::Duration;
use strum::{Display, EnumIter, EnumString};
use supermusr_common::{Channel, DigitizerId};

use crate::{
    messages::{Cache, EventListMessage, FBMessage, TraceMessage},
    Timestamp,
};

pub(crate) use engine::SearchEngine;

#[derive(Default, Clone, EnumString, Display, EnumIter, Copy)]
pub(crate) enum SearchMode {
    #[default]
    ByTimestamp,
    Capture,
    FromEnd,
}

#[derive(Default, Clone, EnumString, Display, EnumIter, Copy)]
pub(crate) enum SearchBy {
    #[default]
    ByChannels,
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
    Halted,
    Successful,
}

#[derive(Default)]
pub(crate) struct SearchResults {
    pub(crate) time: Duration,
    pub(crate) cache: Cache,
}

/*#[derive(Clone)]
pub(crate) enum SearchTarget {
    ByChannel {
        timestamp: Timestamp,
        channels: Vec<Channel>,
        number: usize,
    },
    ByDigitiser {
        timestamp: Timestamp,
        digitiser_ids: Vec<DigitizerId>,
        number: usize,
    },
    FromEnd {
        number: usize,
    }
}*/

#[derive(Default, Clone)]
pub(crate) struct SearchTarget {
    pub(crate) mode: SearchMode,
    pub(crate) timestamp: Timestamp,
    pub(crate) channels: Vec<Channel>,
    pub(crate) digitiser_ids: Vec<DigitizerId>,
    pub(crate) number: usize,
}

impl SearchTarget {
    pub(crate) fn filter_trace_by_channel_and_digtiser_id(&self, msg: &TraceMessage) -> bool {
        //self.channels.
        //    iter()
        //    .any(|&c| msg.has_channel(c)) ||
        self.digitiser_ids
            .iter()
            .any(|&d: &u8| msg.digitiser_id() == d)
    }

    pub(crate) fn filter_eventlist_digtiser_id(&self, msg: &EventListMessage) -> bool {
        self.digitiser_ids
            .iter()
            .any(|&d: &u8| msg.digitiser_id() == d)
    }
}

pub(crate) trait MessageFinder {
    type SearchMode;

    fn init_search(&mut self, target: SearchTarget) -> bool;

    fn status(&mut self) -> Option<SearchStatus>;

    fn results(&mut self) -> Option<SearchResults>;

    async fn update(&mut self);
}
