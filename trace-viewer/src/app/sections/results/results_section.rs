use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::{DisplayErrors, Panel, Section},
        sections::results::{display_trace::DisplayTrace, select_trace::SelectTrace},
        server_functions::FetchSearchSummaries,
    },
    messages::TraceWithEvents,
    structs::TraceSummary,
};

#[component]
pub(crate) fn ResultsSection() -> impl IntoView {
    provide_context(signal::<Option<TraceWithEvents>>(None));

    let fetch_search_summaries =
        use_context::<ServerAction<FetchSearchSummaries>>().expect("This should never fail.");

    move || {
        fetch_search_summaries.value()
        .get()
        .map(|trace_summaries| view!{
            <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors/> }>
                {trace_summaries.map(|trace_summaries| view! { <DisplayResults trace_summaries /> })}
            </ErrorBoundary>
        })
    }
}
#[component]
pub(crate) fn DisplayResults(trace_summaries: Vec<TraceSummary>) -> impl IntoView {
    /*let (selected_message, set_selected_trace) = use_context::<(
        ReadSignal<Option<SelectedTraceIndex>>,
        WriteSignal<Option<SelectedTraceIndex>>,
    )>()
    .expect("");*/
    /*
       Effect::new(move || {
           set_selected_trace.set(selected_message.get().and_then(
               |SelectedTraceIndex { index, channel }| {
                   trace_summaries
                       .get(index)
                       .map(|(metadata, trace)| TraceWithEvents::new(metadata, trace, channel))
               },
           ));
       });
    */
    view! {
        <Section name = "Results" classes = vec!["results"]>
            <Panel>
                <SelectTrace trace_summaries/>
            </Panel>
            <Panel>
                <DisplayTrace />
            </Panel>
        </Section>
    }
}
