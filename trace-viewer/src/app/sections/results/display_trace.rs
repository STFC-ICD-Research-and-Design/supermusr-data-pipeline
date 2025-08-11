use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{components::DisplayErrors, sections::results::results_section::ResultsLevelContext},
    structs::TracePlotly,
};

#[component]
pub(crate) fn DisplayTrace() -> impl IntoView {
    let create_and_fetch_plotly = use_context::<ResultsLevelContext>()
        .expect("ResultsLevelContext should be provided, this should never fail")
        .create_and_fetch_plotly;

    view! {
        <Transition fallback = ||view!("Loading Graph")>
            {move ||create_and_fetch_plotly.value().get().map(|trace| view!{
                <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors /> }>
                    {trace.map(|trace_plotly|
                        view!{ <DisplayGraph trace_plotly /> }
                    )}
                </ErrorBoundary>
            })}
        </Transition>
    }
}

#[component]
pub(crate) fn DisplayGraph(trace_plotly: TracePlotly) -> impl IntoView {
    let TracePlotly {
        title,
        trace_data,
        eventlist_data,
        layout,
    } = trace_plotly;

    let data = eventlist_data
        .map(|eventlist_data| format!("{trace_data}, {eventlist_data}"))
        .unwrap_or(trace_data);

    view! {
        <div class = "trace-graph">
            <div class = "trace-graph-title">
                {title}
            </div>
            <div id="trace-graph" class="plotly-graph-div"></div>
            <script type="text/javascript" inner_html = {format!("
                var data = [{data}];
                var layout = {layout};
                var config = {{ 'scrollZoom': true}};
                Plotly.newPlot('trace-graph', data, layout, config);
            ")}>
            </script>
        </div>
    }
}
