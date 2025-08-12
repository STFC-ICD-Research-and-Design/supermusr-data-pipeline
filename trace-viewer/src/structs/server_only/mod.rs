mod borrowed_messages;
mod search_results;

use std::sync::Arc;

use crate::sessions::SessionEngine;
pub(crate) use borrowed_messages::{
    BorrowedMessageError, EventListMessage, FBMessage, TraceMessage,
};
pub(crate) use search_results::{Cache, SearchResults};
use tokio::sync::Mutex;

/// Encapsulates all run-time settings which are only available to the server.
#[derive(Default, Clone)]
pub struct ServerSideData {
    pub session_engine: Arc<Mutex<SessionEngine>>,
}