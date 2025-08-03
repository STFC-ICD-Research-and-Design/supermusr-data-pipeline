use leptos::{component, prelude::*, view, IntoView};


use crate::app::{components::Panel, sections::DisplaySettingsNodeRefs};

use leptos_chartistry::*;

#[server]
pub async fn create_plotly_on_server(trace: Vec<u16>) -> Result<(String, String), ServerFnError> {
    use tracing::info;
    use plotly::{Plot, Trace, Scatter, common::Mode, layout::Axis, Layout};

    info!("create_plotly_on_server");
    let layout = Layout::new()
        .title(value)
        .x_axis(Axis::new().title("Time (ns)"))
        .y_axis(Axis::new().title("Intensity"));
    let trace = Scatter::new((0..trace.len()).collect::<Vec<_>>(), trace).mode(Mode::Lines);
    Ok((trace.to_json(), layout.to_json()))
}

#[component]
pub(crate) fn Display(
    //selected_trace: impl Fn() -> Option<Vec<u16>> + Send + 'static,
    selected_trace: ReadSignal<Option<Vec<u16>>>,
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
                <Panel>
                    <Suspense fallback = ||view!("Loading Graph")>
                        {move || resource.get().and_then(Result::ok).map(|(data, layout)| view!(
                            <div id="trace-graph" class="plotly-graph-div" style="height:100%; width:100%;"></div>
                            <script type="text/javascript" inner_html = {format!(
                                "Plotly.newPlot( 'trace-graph', {{
                                    'data': {data},
                                    'layout': {layout},
                                    'config':{{scrollZoom: true}}
                                }})"
                            )}>
                            </script>
                        ) )}
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
            }
        })
    }
}
