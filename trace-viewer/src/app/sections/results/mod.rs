mod display;
mod select_trace;

use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::{Panel, Section},
        sections::{results::select_trace::SelectTrace, search::GetSearchResultsServerAction},
    },
    messages::{DigitiserMetadata, DigitiserTrace},
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
pub(crate) fn SearchResults(
    get_search_results_action: GetSearchResultsServerAction,
    set_selected_trace: WriteSignal<Option<Vec<u16>>>,
) -> impl IntoView {
    move || {
        if get_search_results_action.pending().get() {
            view! {
                <Panel>
                    <p> "Searching Broker..."</p>
                </Panel>
            }
            .into_any()
        } else if let Some(search_results) = get_search_results_action.value().get() {
            match search_results {
                Ok(search_results) => {
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

                    let (selected_message, set_selected_message) =
                        signal::<Option<(usize, u32)>>(None);

                    Effect::new(move || {
                        set_selected_trace.set(selected_message.get().and_then(
                            |(index, channel)| {
                                digitiser_messages
                                    .get(index)
                                    .map(|(_, trace)| trace.traces[&channel].clone())
                            },
                        ));
                    });

                    view! {
                        <Section name = "Results" classes = vec!["results"]>
                            <Panel>
                                <SelectTrace trace_summaries selected_message set_selected_message/>
                            </Panel>
                        </Section>
                    }
                    .into_any()
                }
                Err(e) => view! {<div> "Server Error:" {e.to_string()} </div>}.into_any(),
            }
        } else {
            view! {""}.into_any()
        }
    }
}
