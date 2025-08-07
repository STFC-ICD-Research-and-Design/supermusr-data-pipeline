use crate::{
    messages::TraceWithEvents,
    structs::{BrokerInfo, SearchResults, SearchStatus, SearchTarget, SelectedTraceIndex, TraceSummary},
};
use cfg_if::cfg_if;
use leptos::prelude::*;
use tracing::{error, instrument};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{
            DefaultData,
            finder::{MessageFinder, SearchEngine, StatusSharer},
            structs::Topics,
            sessions::SessionEngine,
        };
        use std::sync::{Arc, Mutex};
        use tracing::{debug, info};
        use tokio::sync::mpsc;
    }
}

#[server]
#[instrument(skip_all)]
pub async fn poll_broker(poll_broker_timeout_ms: u64) -> Result<Option<BrokerInfo>, ServerFnError> {
    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    debug!("{default:?}");

    let consumer = supermusr_common::create_default_consumer(
        &default.broker,
        &default.username,
        &default.password,
        &default.consumer_group,
        None,
    )?;

    let searcher = SearchEngine::new(consumer, &default.topics, StatusSharer::new());

    let broker_info = searcher.poll_broker(poll_broker_timeout_ms).await;

    debug!("Literally Finished {broker_info:?}");
    Ok(broker_info)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_new_search(target: SearchTarget) -> Result<String, ServerFnError> {
    debug!("Creating new search task for target: {:?}", target);

    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");

    let uuid = session_engine.create_new_search(
        &default.broker,
        &default.topics,
        &default.consumer_group,
        target,
    )?;

    debug!("New search task has uuid: {}", uuid);

    Ok(uuid)
}

#[server]
pub async fn cancel_search(uuid: String) -> Result<(), ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock()?;
    session_engine.cancel_session(&uuid)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn await_search(uuid: String) -> Result<(), ServerFnError> {
    // Obtain JoinHandle without locking SessionEngine for too long.
    let (handle, is_cancelled) = {
        let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
            .expect("Session engine should be provided, this should never fail.");

        let mut session_engine = session_engine_arc_mutex.lock().expect("");
        let mut session = session_engine.session_mut(&uuid);
        (session.take_search_handle(), session.is_cancelled())
    };

    // Run Future
    tokio::select! {
        results = handle => {
            let results = results
                .inspect(|_| debug!("Successfully found results."))
                .or_else(|e| if e.is_cancelled() { Ok(SearchResults::Cancelled) } else { Err(e) })?;

            // Register results with SessionEngine and return results.
            let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
                .expect("Session engine should be provided, this should never fail.");

            let mut session_engine = session_engine_arc_mutex.lock().expect("");

            let mut session = session_engine.session_mut(&uuid);

            session.register_results(results);
        }
        result = is_cancelled => {
            if let Err(e) = result {
                error!("{}",e);
            }
        }
    }

    Ok(())
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn fetch_search_summaries(uuid: String) -> Result<Option<Vec<TraceSummary>>, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let session_engine = session_engine_arc_mutex.lock().expect("");
    let session = session_engine.session(&uuid);

    Ok(session.get_search_summaries())
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn fetch_selected_trace(uuid: String, index_and_channel: SelectedTraceIndex) -> Result<Option<TraceWithEvents>, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");
    let mut session = session_engine.session(&uuid);
    session.get_selected_trace(index_and_channel)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_and_fetch_plotly_of_selected_trace(uuid: String, index_and_channel: SelectedTraceIndex) -> Result<(String, Option<String>, String), ServerFnError> {
    create_plotly_on_server(fetch_selected_trace(uuid, index_and_channel).await?.expect("")).await
}

#[server]
pub async fn fetch_status(uuid: String) -> Result<SearchStatus, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock()?;
    session_engine.get_session_status(&uuid).await
}

#[server]
pub async fn create_plotly_on_server(
    trace_with_events: TraceWithEvents,
) -> Result<(String, Option<String>, String), ServerFnError> {
    use plotly::{
        Layout, Scatter, Trace,
        color::NamedColor,
        common::Mode,
        common::{Line, Marker},
        layout::Axis,
    };

    info!("create_plotly_on_server");
    let layout = Layout::new()
        .title("Trace and Eventlist")
        .show_legend(false)
        .auto_size(false)
        .x_axis(Axis::new().title("Time (ns)"))
        .y_axis(Axis::new().title("Intensity"));
    let trace = Scatter::new(
        (0..trace_with_events.trace.len()).collect::<Vec<_>>(),
        trace_with_events.trace,
    )
    .mode(Mode::Lines)
    .name("Trace")
    .line(Line::new().color(NamedColor::CadetBlue));
    let eventlist = trace_with_events.eventlist.map(|eventlist| {
        Scatter::new(
            eventlist.iter().map(|event| event.time).collect::<Vec<_>>(),
            eventlist
                .iter()
                .map(|event| event.intensity)
                .collect::<Vec<_>>(),
        )
        .mode(Mode::Markers)
        .name("Events")
        .marker(Marker::new().color(NamedColor::IndianRed))
    });
    Ok((
        trace.to_json(),
        eventlist.as_deref().map(Trace::to_json),
        layout.to_json(),
    ))
}
