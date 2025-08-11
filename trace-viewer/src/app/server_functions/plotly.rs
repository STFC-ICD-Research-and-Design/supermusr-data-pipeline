use crate::structs::{SelectedTraceIndex, TracePlotly};
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

    let (metadata, digitiser_traces) = session_engine
        .session(&uuid)?
        .get_selected_trace(index_and_channel.index)?;

    let trace = digitiser_traces
        .traces
        .get(&index_and_channel.channel)
        .ok_or(SessionError::ChannelNotFound)?;

    let eventlist = digitiser_traces
        .events
        .as_ref()
        .map(|events| {
            events
                .get(&index_and_channel.channel)
                .ok_or(SessionError::ChannelNotFound)
        })
        .transpose()?;

    create_plotly(metadata, index_and_channel.channel, trace, eventlist)
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::{
            app::{ServerError, SessionError},
            structs::{DigitiserMetadata, Trace as MuonTrace, EventList},
            Channel,
            sessions::SessionEngine
        };
        use plotly::{
            Layout, Scatter, Trace,
            color::NamedColor,
            common::Mode,
            common::{Line, Marker},
            layout::Axis,
        };
        use std::sync::{Arc, Mutex};
        use tracing::{debug, error, info};

        fn create_plotly<'a>(metadata: &DigitiserMetadata, channel: Channel, trace: &'a MuonTrace, eventlist: Option<&'a EventList>) -> Result<TracePlotly, ServerFnError> {
            info!("create_plotly_on_server");

            let layout = Layout::new()
                .title("Trace and Eventlist")
                .show_legend(false)
                .auto_size(false)
                .x_axis(Axis::new().title("Time (ns)"))
                .y_axis(Axis::new().title("Intensity"));

            let trace = Scatter::new(
                (0..trace.len()).collect::<Vec<_>>(),
                trace.clone(),
            )
            .mode(Mode::Lines)
            .name("Trace")
            .line(Line::new().color(NamedColor::CadetBlue));

            let eventlist = eventlist.map(|eventlist| {
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
                title: format!("Channel {} from Digitiser {}", channel, metadata.id),
                trace_data: trace.to_json(),
                eventlist_data: eventlist.as_deref().map(Trace::to_json),
                layout: layout.to_json(),
            })
        }
    }
}
