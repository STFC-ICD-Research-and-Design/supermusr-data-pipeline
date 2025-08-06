mod search_engine;
mod searcher;
mod task;

use crate::structs::{BrokerInfo, SearchResults, SearchTarget};

pub use search_engine::SearchEngine;

pub use task::StatusSharer;

pub(crate) trait MessageFinder {
    type SearchMode;

    async fn search(&mut self, target: SearchTarget) -> SearchResults;

    async fn poll_broker(&self, poll_broker_timeout_ms: u64) -> Option<BrokerInfo>;
}
