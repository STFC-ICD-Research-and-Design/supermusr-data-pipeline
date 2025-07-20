use leptos::{component, prelude::*, view, IntoView};

use crate::app::components::{ControlBoxWithLabel, Panel, VerticalBlock};

//use leptos_chart::*;
//use leptos_chartistry::*;

#[component]
pub(crate) fn Display() -> impl IntoView {
    /*let chart = leptos_chart::Cartesian::new(
        leptos_chart::Series::from(vec![1.0, 6.0, 9.]),
        leptos_chart::Series::from(vec![1.0, 3.0, 5.])
    )
        .set_view(820, 620, 3, 100, 100, 20);
    let color = leptos_chart::Color::from("#ff0000");*/

    view! {
        <Panel>
            <VerticalBlock>
                <div></div>
                // color is option
                    //<LineChart chart = chart \>
                /*<Chart
                    // Sets the width and height
                    aspect_ratio=AspectRatio::from_outer_ratio(600.0, 300.0)

                    // Decorate our chart
                    top=RotatedLabel::middle("My garden")
                    left=TickLabels::aligned_floats()
                    right=Legend::end()
                    bottom=TickLabels::timestamps()
                    inner=[
                        AxisMarker::left_edge().into_inner(),
                        AxisMarker::bottom_edge().into_inner(),
                        XGridLine::default().into_inner(),
                        YGridLine::default().into_inner(),
                        XGuideLine::over_data().into_inner(),
                        YGuideLine::over_mouse().into_inner(),
                    ]
                    tooltip=Tooltip::left_cursor()

                    // Describe the data
                    //series=Series::new(|data: &MyData| data.x)
                    //    .line(Line::new(|data: &MyData| data.y1).with_name("butterflies"))
                    //   .line(Line::new(|data: &MyData| data.y2).with_name("dragonflies"))
                    //data=data
                />*/
                <div class = "chart-area">
                </div>
                <div>
                    <ControlBoxWithLabel name = "width" label = "Width (px):">
                        <input type = "number" />
                    </ControlBoxWithLabel>
                    <ControlBoxWithLabel name = "height" label = "Height (px):">
                        <input type = "number" />
                    </ControlBoxWithLabel>
                </div>
            </VerticalBlock>
        </Panel>
    }
}