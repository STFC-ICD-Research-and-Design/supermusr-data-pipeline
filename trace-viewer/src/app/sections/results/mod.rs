mod display;
mod select_trace;

use leptos::{IntoView, component, either::Either, prelude::*, view};

use crate::{
    app::{
        components::{DisplayErrors, Panel, Section},
        sections::results::select_trace::SelectTrace,
        server_functions::{FetchSearchSummaries, FetchSelectedTrace},
    },
    messages::{DigitiserMetadata, DigitiserTrace, TraceWithEvents},
    structs::{SearchResults, SelectedTraceIndex, TraceSummary},
};

pub(crate) use display::Display;

#[component]
pub(crate) fn Results() -> impl IntoView {
    provide_context(signal::<Option<TraceWithEvents>>(None));

    let fetch_search_summaries =
        use_context::<ServerAction<FetchSearchSummaries>>().expect("This should never fail.");

    move || {
        fetch_search_summaries.value()
        .get()
        .map(|trace_summaries| view!{
            <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors/> }>
                {trace_summaries.map(|trace_summaries| view! { <DisplayResultsOption trace_summaries /> })}
            </ErrorBoundary>
        }
    )
    }
}

#[component]
pub(crate) fn DisplayResultsOption(trace_summaries: Option<Vec<TraceSummary>>) -> impl IntoView {
    match trace_summaries {
        Some(trace_summaries) => Either::Left(view! { <DisplayResults trace_summaries /> }),
        None => Either::Right(view! { <div> "No results." </div> }),
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
                <Display />
            </Panel>
        </Section>
    }
}
