//! Contains structs used to search the broker for messages, as well as poll it for its contents.
//!
//! This module is only included in the server build.
mod search_engine;
mod status_sharer;
mod task;
mod topic_searcher;

pub(crate) use search_engine::{SearchEngine, SearchEngineError};
pub(crate) use status_sharer::StatusSharer;
