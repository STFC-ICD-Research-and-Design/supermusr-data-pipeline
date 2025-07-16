mod search_engine;
mod searcher;
mod task;

use crate::{messages::{Cache, FBMessage}, Timestamp, structs::{BrokerInfo, SearchResults, SearchTarget}};
use chrono::TimeDelta;
use supermusr_common::{Channel, DigitizerId};

pub use search_engine::SearchEngine;
pub(crate) trait MessageFinder {
    type SearchMode;

    async fn search(&mut self, target : SearchTarget) -> SearchResults;
    
    async fn poll_broker<'a, M: FBMessage<'a>>(&self, poll_broker_timeout_ms: u64,) -> Option<BrokerInfo>;
/*
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
     */
}
