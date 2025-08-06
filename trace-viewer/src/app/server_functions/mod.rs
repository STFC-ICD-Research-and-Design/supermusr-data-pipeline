use leptos::prelude::*;
use cfg_if::cfg_if;
use tracing::instrument;
use crate::{messages::TraceWithEvents, structs::{BrokerInfo, SearchResults, SearchStatus, SearchTarget}};

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
pub async fn poll_broker(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    poll_broker_timeout_ms: u64,
) -> Result<Option<BrokerInfo>, ServerFnError> {
    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    //let status = use_context::<Arc<Mutex<SearchStatus>>>().expect("");

    debug!("{default:?}");

    let consumer = supermusr_common::create_default_consumer(
        &broker,
        &default.username,
        &default.password,
        &consumer_group,
        None,
    )?;

    let searcher = SearchEngine::new(
        consumer,
        &Topics {
            trace_topic,
            digitiser_event_topic,
        },
        StatusSharer::new(),
    );

    let broker_info = searcher.poll_broker(poll_broker_timeout_ms).await;

    debug!("Literally Finished {broker_info:?}");
    Ok(broker_info)
}



#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_new_search(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    target: SearchTarget,
) -> Result<String, ServerFnError> {

    debug!("Creating new search task for target: {:?}", target);

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");

    let uuid = session_engine.create_new_search(
        broker,
        trace_topic,
        digitiser_event_topic,
        consumer_group,
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
    session_engine.cancel_session(&uuid).await
}


#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn get_search_results(uuid: String) -> Result<Option<SearchResults>, ServerFnError> {

    // Obtain JoinHandle without locking SessionEngine for too long.
    let handle = {
        let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
            .expect("Session engine should be provided, this should never fail.");

        let mut session_engine = session_engine_arc_mutex.lock().expect("");
        let mut session = session_engine.session_mut(&uuid);

        session.take_search_handle()
    };

    // Run Future
    let results = handle.await
        .inspect(|_|debug!("Successfully found results."))
        .map(Some)
        .or_else(|e|if e.is_cancelled() { Ok(None) } else { Err(e) })?;
    
    // Register results with SessionEngine and return results.
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");

    let mut session = session_engine.session_mut(&uuid);

    session.register_results(results);

    Ok(session.get_search_results())
}

#[server]
pub async fn get_status(uuid: String) -> Result<SearchStatus, ServerFnError> {

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock()?;
    session_engine.get_session_status(&uuid).await
}

#[server]
pub async fn create_plotly_on_server(trace_with_events: TraceWithEvents) -> Result<(String, Option<String>, String), ServerFnError> {
    use plotly::{Trace, Scatter, common::Mode, layout::Axis, Layout, color::NamedColor, common::{Line, Marker}};

    info!("create_plotly_on_server");
    let layout = Layout::new()
        .title("Trace and Eventlist")
        .show_legend(false)
        .auto_size(false)
        .x_axis(Axis::new().title("Time (ns)"))
        .y_axis(Axis::new().title("Intensity"));
    let trace = Scatter::new(
            (0..trace_with_events.trace.len()).collect::<Vec<_>>(),
            trace_with_events.trace
        )
        .mode(Mode::Lines)
        .name("Trace")
        .line(Line::new().color(NamedColor::CadetBlue));
    let eventlist = trace_with_events.eventlist.map(|eventlist|
        Scatter::new(
            eventlist.iter().map(|event|event.time).collect::<Vec<_>>(),
            eventlist.iter().map(|event|event.intensity).collect::<Vec<_>>(),
        )
        .mode(Mode::Markers)
        .name("Events")
        .marker(Marker::new().color(NamedColor::IndianRed))
    );
    Ok((trace.to_json(), eventlist.as_deref().map(Trace::to_json), layout.to_json()))
}
