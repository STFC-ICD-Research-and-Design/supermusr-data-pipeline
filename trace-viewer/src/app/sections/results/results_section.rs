use crate::{
    app::{
        components::{DisplayErrors, Section},
        main_content::MainLevelContext,
        sections::results::{
            context::ResultsLevelContext, display_trace_graph::DisplayTrace,
            search_results::SearchResultsPanel,
        },
        server_functions::CreateAndFetchPlotly,
    },
    structs::SearchSummary,
};
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn ResultsSection() -> impl IntoView {
    let main_context = use_context::<MainLevelContext>()
        .expect("MainLevelContext should be provided, this should never fail.");
    let fetch_search_summaries = main_context.fetch_search_search;

    // Currently Selected Digitiser Trace Message
    provide_context(ResultsLevelContext {
        create_and_fetch_plotly: ServerAction::<CreateAndFetchPlotly>::new(),
        selected_channels_only: RwSignal::new(false),
    });

    move || {
        fetch_search_summaries.value()
        .get()
        .map(|search_summary| view!{
            <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors/> }>
                {search_summary.map(|search_summary| view! { <DisplayResults search_summary /> })}
            </ErrorBoundary>
        })
    }
}

#[component]
pub(crate) fn DisplayResults(search_summary: SearchSummary) -> impl IntoView {
    view! {
        <Section id = "results" text = "Results">
            <SearchResultsPanel search_summary/>
            <DisplayTrace />
        </Section>
    }
}
