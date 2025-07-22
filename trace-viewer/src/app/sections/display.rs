use leptos::{component, html::Input, prelude::*, view, IntoView};

use crate::{app::components::{ControlBoxWithLabel, Panel, VerticalBlock}, messages::{DigitiserMetadata, DigitiserTrace}};

use leptos_chartistry::*;

#[component]
pub(crate) fn Display(selected_trace: impl Fn() -> Option<Vec<u16>> + Send + 'static, width_ref: NodeRef<Input>, height_ref: NodeRef<Input>) -> impl IntoView {
    move ||
    selected_trace().map(|trace| {
        let data = Signal::derive(move ||trace.iter().enumerate().map(|(x,y)|(x as f64, *y as f64)).collect::<Vec<_>>());
        let width = width_ref.get().and_then(|node_ref|node_ref.value().parse().ok()).unwrap_or(800.0);
        let height = height_ref.get().and_then(|node_ref|node_ref.value().parse().ok()).unwrap_or(600.0);
        view! {
            <Panel>
                <VerticalBlock>
                    <div></div>
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
                            series = Series::new(|&(x,y)|x)
                                .line(Line::new(|&(x,y)|y)
                                .with_name("trace"))
                            data = data
                        />
                    </div>
                </VerticalBlock>
            </Panel>
        }
    })
}