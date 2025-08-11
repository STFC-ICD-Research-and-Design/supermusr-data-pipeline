use crate::structs::{SearchTarget, TraceSummary};
use cfg_if::cfg_if;
use leptos::prelude::*;
use tracing::instrument;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{
            app::server_functions::errors::ServerError,
            sessions::SessionEngine,
            structs::SearchResults,
        };
        use std::sync::{Arc, Mutex};
        use tracing::{debug, error, info};
    }
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_new_search(target: SearchTarget) -> Result<String, ServerFnError> {
    debug!("Creating new search task for target: {:?}", target);

    // The mutex should be in scope to apply a lock.
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    let uuid = session_engine.create_new_search(target)?;

    debug!("New search task has uuid: {}", uuid);

    Ok(uuid)
}

#[server]
pub async fn cancel_search(uuid: String) -> Result<(), ServerFnError> {
    // The mutex should be in scope to apply a lock.
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");
    let mut session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    session_engine.cancel_session(&uuid)?;
    Ok(())
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn await_search(uuid: String) -> Result<String, ServerFnError> {
    use crate::sessions::SessionSearchBody;

    // Obtain SessionSearchBody without locking SessionEngine for too long.
    let SessionSearchBody {
        handle,
        cancel_recv,
    } = {
        let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
            .expect("Session engine should be provided, this should never fail.");

        let mut session_engine = session_engine_arc_mutex
            .lock()
            .map_err(|e| ServerError::CannotObtainSessionEngine)?;

        let mut session = session_engine.session_mut(&uuid)?;
        session.take_search_body()?
    };

    // Run Future
    tokio::select! {
        results = handle => {
            let results = results
                .inspect(|_| debug!("Successfully found results."))
                .or_else(|e| if e.is_cancelled() { Ok(Ok(SearchResults::Cancelled)) } else { Err(e) })??;

            // Register results with SessionEngine and return results.
            let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
                .expect("Session engine should be provided, this should never fail.");

            let mut session_engine = session_engine_arc_mutex
                .lock()
                .map_err(|e| ServerError::CannotObtainSessionEngine)?;

            session_engine.session_mut(&uuid)?
                .register_results(results);
        }
        result = cancel_recv => {
            if let Err(e) = result {
                error!("{}",e);
            }
        }
    }

    Ok(uuid)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn fetch_search_summaries(uuid: String) -> Result<Vec<TraceSummary>, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|e| ServerError::CannotObtainSessionEngine)?;

    let session = session_engine.session(&uuid)?;

    Ok(session.get_search_summaries()?)
}
