use leptos::{IntoView, component, prelude::*, view};

use crate::app::{components::DisplayErrors, server_functions::CreateAndFetchPlotlyOfSelectedTrace};

#[component]
pub(crate) fn Display() -> impl IntoView {
    //let node_refs = use_context::<DisplaySettingsNodeRefs>().expect("");
    let create_and_fetch_plotly_of_selected_trace = use_context::<ServerAction<CreateAndFetchPlotlyOfSelectedTrace>>().expect("");
    view! {
        <Transition fallback = ||view!("Loading Graph")>
            {move ||create_and_fetch_plotly_of_selected_trace.value().get().map(|trace| view!{
                <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors /> }>
                    {trace.map(|(trace_data, eventlist_data, layout)|
                        view!{ <DisplayGraph trace_data eventlist_data layout /> }
                    )}
                </ErrorBoundary>
            })}
        </Transition>
    }
}

#[component]
pub(crate) fn DisplayGraph(trace_data: String, eventlist_data: Option<String>, layout: String) -> impl IntoView {
    let data = eventlist_data.map(|eventlist_data|format!("{trace_data}, {eventlist_data}"))
        .unwrap_or(trace_data);
        
    view!{
        <h2>
        "Channel something of digitiser something "
        //"Channel " {trace.channel} " of Digitiser " {trace.metadata.id}
        </h2>
        <div id="trace-graph" class="plotly-graph-div"></div>
        <script type="text/javascript" inner_html = {format!("
            var data = [{data}];
            var layout = {layout};
            var config = {{ 'scrollZoom': true}};
            Plotly.newPlot('trace-graph', data, layout, config);
        ")}>
        </script>
    }
}