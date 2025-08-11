//! All server functions appear here.
//!
mod errors;
mod plotly;
mod search;

use crate::structs::{BrokerInfo, ClientSideData, SearchStatus};
use cfg_if::cfg_if;
use leptos::prelude::*;
use tracing::instrument;

pub use plotly::CreateAndFetchPlotly;
pub use search::{AwaitSearch, CancelSearch, CreateNewSearch, FetchSearchSummaries};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{
            structs::SearchResults,
            sessions::SessionEngine,
        };
        use std::sync::{Arc, Mutex};
        use tracing::{debug, error, info};
        pub(crate) use errors::{SessionError, ServerError};
    }
}

#[server]
#[instrument(skip_all)]
pub async fn get_client_side_data() -> Result<ClientSideData, ServerFnError> {
    // The mutex should be in scope to apply a lock.
    Ok(use_context::<ClientSideData>()
        .expect("Client-side data should be provided, this should never fail."))
}

#[server]
#[instrument(skip_all)]
pub async fn poll_broker(poll_broker_timeout_ms: u64) -> Result<BrokerInfo, ServerFnError> {
    // The mutex should be in scope to apply a lock.
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    let broker_info = session_engine.poll_broker(poll_broker_timeout_ms).await?;

    Ok(broker_info)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn refresh_session(uuid: String) -> Result<(), ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    let mut session = session_engine.session_mut(&uuid)?;
    session.refresh();
    debug!("Session {uuid} refreshed.");
    Ok(())
}

#[server]
pub async fn fetch_status(uuid: String) -> Result<SearchStatus, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    Ok(session_engine.get_session_status(&uuid).await?)
}
