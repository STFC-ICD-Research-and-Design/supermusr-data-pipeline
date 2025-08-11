use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::{DisplayErrors, Section},
        main_content::MainLevelContext,
        sections::results::{display_trace::DisplayTrace, select_trace::SelectTrace},
        server_functions::{CreateAndFetchPlotly, FetchSearchSummaries},
    },
    structs::TraceSummary,
};

#[derive(Clone)]
pub(super) struct ResultsLevelContext {
    pub(super) create_and_fetch_plotly: ServerAction<CreateAndFetchPlotly>,
}

#[component]
pub(crate) fn ResultsSection() -> impl IntoView {
    let main_context = use_context::<MainLevelContext>()
        .expect("MainLevelContext should be provided, this should never fail.");
    let fetch_search_summaries = main_context.fetch_search_search;

    // Currently Selected Digitiser Trace Message
    provide_context(ResultsLevelContext {
        create_and_fetch_plotly: ServerAction::<CreateAndFetchPlotly>::new(),
    });

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
