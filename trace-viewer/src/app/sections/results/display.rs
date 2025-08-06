use leptos::{component, prelude::*, view, IntoView};

use crate::{app::{sections::DisplaySettingsNodeRefs, server_functions::create_plotly_on_server}, messages::TraceWithEvents};

#[component]
pub(crate) fn Display(
    selected_trace: ReadSignal<Option<TraceWithEvents>>,
) -> impl IntoView {
    //let node_refs = use_context::<DisplaySettingsNodeRefs>().expect("");
    move || {
        selected_trace.get().map(|trace| {
            let resource = Resource::new(move ||trace.clone(), |trace| {
                create_plotly_on_server(trace)
            });
            view! {
                <Suspense fallback = ||view!("Loading Graph")>
                    {move || resource.get()
                            .and_then(Result::ok)
                            .map(|(trace_data, eventlist_data, layout)| {

                                let data = eventlist_data.map(|eventlist_data|format!("{trace_data}, {eventlist_data}"))
                                    .unwrap_or(trace_data);
                                
                                view!(
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
                            )}
                        )
                    }
                </Suspense>
            }
        })
    }
}
