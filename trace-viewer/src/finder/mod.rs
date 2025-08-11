//! Contains structs used to search the broker for messages, as well as poll it for its contents.
//! 
//! This module is only included in the server build.
//! 
mod search_engine;
mod topic_searcher;
mod status_sharer;
mod task;

pub use search_engine::{SearchEngine, SearchEngineError};
pub use status_sharer::StatusSharer;
