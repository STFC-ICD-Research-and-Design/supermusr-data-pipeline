use crate::structs::{
    SelectedTraceIndex, TracePlotly
};
use cfg_if::cfg_if;
use leptos::prelude::*;
use tracing::instrument;

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_and_fetch_plotly(
    uuid: String,
    index_and_channel: SelectedTraceIndex,
) -> Result<TracePlotly, ServerFnError> {
    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let session_engine = session_engine_arc_mutex
        .lock()
        .map_err(|_| ServerError::CannotObtainSessionEngine)?;

    let trace = session_engine
        .session(&uuid)?
        .get_selected_trace(index_and_channel)?;

    create_plotly(trace)
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{app::server_functions::ServerError, sessions::SessionEngine, structs::TraceWithEvents};
            use plotly::{
                Layout, Scatter, Trace,
                color::NamedColor,
                common::Mode,
                common::{Line, Marker},
                layout::Axis,
            };
        use std::sync::{Arc, Mutex};
        use tracing::{debug, error, info};

        fn create_plotly(trace_with_events: TraceWithEvents) -> Result<TracePlotly, ServerFnError> {
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

            Ok(TracePlotly {
                title: format!("Channel {} from Digitiser {}", trace_with_events.channel,trace_with_events.metadata.id),
                trace_data: trace.to_json(),
                eventlist_data: eventlist.as_deref().map(Trace::to_json),
                layout: layout.to_json(),
            })
        }
    }
}