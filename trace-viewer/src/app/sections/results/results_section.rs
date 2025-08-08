use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::{DisplayErrors, Section},
        sections::results::{display_trace::DisplayTrace, select_trace::SelectTrace},
        server_functions::FetchSearchSummaries,
    },
    messages::TraceWithEvents,
    structs::TraceSummary,
};

#[component]
pub(crate) fn ResultsSection() -> impl IntoView {
    provide_context(signal::<Option<TraceWithEvents>>(None));

    let fetch_search_summaries = use_context::<ServerAction<FetchSearchSummaries>>()
        .expect("FetchSearchSummaries should be provided, this should never fail.");

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
    view! {
        <Section id = "results" text = "Results">
            <SelectTrace trace_summaries/>
            <DisplayTrace />
        </Section>
    }
}
