use crate::{
    app::{
        components::{DisplayErrors, Section},
        main_content::MainLevelContext,
        sections::results::{
            display_trace::DisplayTrace, context::ResultsLevelContext,
            results_settings::DisplayType, select_trace::SelectTrace,
        },
        server_functions::CreateAndFetchPlotly,
    },
    structs::TraceSummary,
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
        display_mode: RwSignal::new(DisplayType::Single),
        display_all_channels: RwSignal::new(true),
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
