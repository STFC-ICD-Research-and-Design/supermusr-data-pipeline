use leptos::{IntoView, component, prelude::*, view};

use crate::app::{components::Panel, sections::DisplaySettingsNodeRefs};

use leptos_chartistry::*;

#[server]
pub async fn create_plotly_on_server() -> Result<String, ServerFnError> {
    use plotly::{Plot, Scatter};

    let mut plot = Plot::new();
    let trace = Scatter::new(vec![0, 1, 2], vec![2, 1, 0]);
    plot.add_trace(trace);

    Ok(plot.to_inline_html(None))
}

#[component]
pub(crate) fn Display(
    //selected_trace: impl Fn() -> Option<Vec<u16>> + Send + 'static,
    selected_trace: ReadSignal<Option<Vec<u16>>>,
) -> impl IntoView {
    let node_refs = use_context::<DisplaySettingsNodeRefs>().expect("");
    move || {
        selected_trace.get().map(|trace| {
            let data = Signal::derive(move || {
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
                .unwrap_or(600.0);

            let resource = Resource::new(||(), |_| {
                create_plotly_on_server()
            });
            view! {
                <Panel>
                    <div>
                        {move || resource.get()}
                    </div>
                    <div class = "chart-area">
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
                    </div>
                </Panel>
            }
        })
    }
}
