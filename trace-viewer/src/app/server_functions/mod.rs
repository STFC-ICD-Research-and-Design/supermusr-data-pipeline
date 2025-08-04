use leptos::prelude::*;

use tracing::instrument;

use crate::{messages::TraceWithEvents, structs::{BrokerInfo, SearchResults, SearchStatus, SearchTarget}};

#[server]
#[instrument(skip_all)]
pub async fn poll_broker(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    poll_broker_timeout_ms: u64,
) -> Result<Option<BrokerInfo>, ServerFnError> {
    use crate::{
        DefaultData,
        finder::{MessageFinder, SearchEngine},
        structs::Topics,
    };
    //use std::sync::{Arc, Mutex};
    use tracing::debug;

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
        //status,
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
    use crate::sessions::SessionEngine;
    use std::sync::{Arc, Mutex};
    use tracing::debug;

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
#[instrument(skip_all, err(level = "warn"))]
pub async fn get_search_results(uuid: String) -> Result<SearchResults, ServerFnError> {
    use crate::sessions::SessionEngine;
    use std::sync::{Arc, Mutex};

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");
    Ok(session_engine.get_search_results(&uuid).await)
}

#[server]
pub async fn get_status(old_status: SearchStatus, uuid: String) -> Result<SearchStatus, ServerFnError> {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;
    use crate::sessions::SessionEngine;

    loop {
        let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
            .expect("Session engine should be provided, this should never fail.");

        let mut session_engine = session_engine_arc_mutex.lock()?;
        let new_status = session_engine.get_session_status(&uuid)?;
        if new_status != old_status {
            return Ok(new_status);
        }
        let _ = sleep(Duration::from_millis(100));
    }
}

#[server]
pub async fn create_plotly_on_server(trace_with_events: TraceWithEvents) -> Result<(String, Option<String>, String), ServerFnError> {
    use tracing::info;
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
