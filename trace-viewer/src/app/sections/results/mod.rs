mod display;
mod select_trace;
mod statusbar;

use leptos::{component, either::Either, prelude::*, view, IntoView};

use crate::{
    app::{
        components::{DisplayErrors, Panel, Section},
        sections::results::{select_trace::SelectTrace, statusbar::Statusbar}, server_functions::GetSearchResults,
    },
    messages::{DigitiserMetadata, DigitiserTrace, TraceWithEvents}, structs::SearchResults,
};

pub(crate) use display::Display;
use select_trace::TraceSummary;

fn extract_summary(
    (index, (metadata, trace)): (usize, &(DigitiserMetadata, DigitiserTrace)),
) -> TraceSummary {
    let date = metadata
        .timestamp
        .date_naive()
        .format("%y-%m-%d")
        .to_string();
    let time = metadata.timestamp.time().format("%H:%M:%S.%f").to_string();
    let id = metadata.id;
    let channels = trace.traces.keys().copied().collect::<Vec<_>>();
    TraceSummary {
        date,
        time,
        index,
        id,
        channels,
    }
}

#[component]
pub(crate) fn Results() -> impl IntoView {

    let (selected_message, set_selected_message) = signal::<Option<(usize, u32)>>(None);
    let (selected_trace, set_selected_trace) = signal::<Option<TraceWithEvents>>(None);

    provide_context((selected_message, set_selected_message));
    provide_context((selected_trace, set_selected_trace));

    let get_search_results_action = use_context::<ServerAction<GetSearchResults>>().expect("This should never fail.");

    move || {
        if get_search_results_action.pending().get() {
            set_selected_message.set(None);
            Either::Left(
                view! {
                    <Section name = "Results" classes = vec!["getting-results"]>
                        <Panel>
                            <Statusbar />
                        </Panel>
                    </Section>
                }
            )
        } else {
            Either::Right(
                get_search_results_action.value()
                    .get()
                    .map(|search_results| view!{
                        <ErrorBoundary fallback = |errors| view!{ <DisplayErrors errors/> }>
                            {search_results.map(|search_results| view! { <DisplayResultsOption search_results /> })}
                        </ErrorBoundary>
                    }
                )
            )
        }
    }
}

#[component]
pub(crate) fn DisplayResultsOption(search_results: Option<SearchResults>) -> impl IntoView {
    match search_results {
        Some(search_results) => Either::Left(
            view!{
                <DisplayResults search_results />
            }
        ),
        None => Either::Right(
            view!{
                <div> "No results." </div>
            }
        ),
    }
}

#[component]
pub(crate) fn DisplayResults(search_results: SearchResults) -> impl IntoView {
    let (selected_message, set_selected_message) = use_context::<(ReadSignal<Option<(usize, u32)>>, WriteSignal<Option<(usize, u32)>>)>().expect("");
    let (selected_trace, set_selected_trace) = use_context::<(ReadSignal<Option<TraceWithEvents>>, WriteSignal<Option<TraceWithEvents>>)>().expect("");

    let mut digitiser_messages = search_results
        .cache
        .iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    digitiser_messages.sort_by(|(metadata1, _), (metadata2, _)| {
        metadata1.timestamp.cmp(&metadata2.timestamp)
    });

    let trace_summaries = digitiser_messages
        .iter()
        .enumerate()
        .map(extract_summary)
        .collect::<Vec<_>>();

    Effect::new(move || {
        set_selected_trace.set(selected_message.get().and_then(
            |(index, channel)| {
                digitiser_messages
                    .get(index)
                    .map(|(metadata, trace)|
                        TraceWithEvents::new(metadata, trace, channel)
                    )
            }
        ));
    });

    view! {
        <Section name = "Results" classes = vec!["results"]>
            <Panel>
                <SelectTrace trace_summaries/>
            </Panel>
            <Panel>
                <Display selected_trace />
            </Panel>
        </Section>
    }
}