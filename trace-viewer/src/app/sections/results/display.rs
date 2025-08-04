use leptos::{component, prelude::*, view, IntoView};

use crate::{app::{components::{Panel, Section}, sections::DisplaySettingsNodeRefs}, messages::TraceWithEvents};

use leptos_chartistry::*;

#[server]
pub async fn create_plotly_on_server(trace_with_events: TraceWithEvents) -> Result<(String, Option<String>, String), ServerFnError> {
    use tracing::info;
    use plotly::{Trace, Scatter, common::Mode, layout::{Axis, Legend}, Layout, color::NamedColor, common::{Line, Marker}};

    info!("create_plotly_on_server");
    let layout = Layout::new()
        .title("Trace and Eventlist")
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

#[component]
pub(crate) fn Display(
    //selected_trace: impl Fn() -> Option<Vec<u16>> + Send + 'static,
    selected_trace: ReadSignal<Option<TraceWithEvents>>,
) -> impl IntoView {
    //let node_refs = use_context::<DisplaySettingsNodeRefs>().expect("");
    move || {
        selected_trace.get().map(|trace| {
            /*let data = Signal::derive(move || {
                trace
                    .iter()
                    .enumerate()
                    .map(|(x, y)| (x as f64, *y as f64))
                    .collect::<Vec<_>>()
            });
            let width = node_refs
                .width_ref
                .get()
                .and_then(|node_ref| node_ref.value().parse().ok())
                .unwrap_or(800.0);
            let height = node_refs
                .height_ref
                .get()
                .and_then(|node_ref| node_ref.value().parse().ok())
                .unwrap_or(600.0);*/

            let resource = Resource::new(move ||trace.clone(), |trace| {
                create_plotly_on_server(trace)
            });
            view! {
                <Section name = "Trace Graph">
                <Panel>
                    <Suspense fallback = ||view!("Loading Graph")>
                        {move || resource.get().and_then(Result::ok).map(|(trace_data, eventlist_data, layout)| {
                            let data = eventlist_data.map(|eventlist_data|format!("{trace_data}, {eventlist_data}"))
                                .unwrap_or(trace_data);
                            view!(
                                <h2>
                                "Channel something of digitiser something "
                                //"Channel " {trace.channel} " of Digitiser " {trace.metadata.id}
                                </h2>
                                <div id="trace-graph" class="plotly-graph-div" style="height:100%; width:100%;"></div>
                                <script type="text/javascript" inner_html = {format!("
                                    var data = [{data}];
                                    var layout = {layout};
                                    var config = {{ 'scrollZoom': true}};
                                    Plotly.newPlot('trace-graph', data, layout, config);
                                ")}>
                                </script>
                            )}
                        )}
                    </Suspense>
                    /*<div class = "chart-area">
                        <Chart
                            // Sets the width and height
                            aspect_ratio=AspectRatio::from_outer_ratio(width, height)

                            // Decorate our chart
                            top=RotatedLabel::middle("My garden")
                            left=TickLabels::aligned_floats()
                            right=Legend::end()
                            bottom=TickLabels::aligned_floats()
                            inner=[
                                AxisMarker::left_edge().into_inner(),
                                AxisMarker::bottom_edge().into_inner(),
                                XGridLine::default().into_inner(),
                                YGridLine::default().into_inner(),
                                XGuideLine::over_data().into_inner(),
                                YGuideLine::over_mouse().into_inner(),
                            ]
                            tooltip = Tooltip::left_cursor()

                            // Describe the data
                            series = Series::new(|&(x,_)|x)
                                .line(Line::new(|&(_,y)|y)
                                .with_name("trace"))
                            data = data
                        />
                    </div>*/
                </Panel>
                </Section>
            }
        })
    }
}
