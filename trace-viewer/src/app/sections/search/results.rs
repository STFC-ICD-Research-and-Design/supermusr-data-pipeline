use leptos::{component, view, IntoView, prelude::*};

use crate::{app::{components::{Panel, Section}, sections::Display}, structs::SearchResults};

#[component]
pub(crate) fn Results(search_results: Result<SearchResults, ServerFnError>) -> impl IntoView {
    
    match search_results {
        Ok(search_results) => {
            let result_summaries = search_results.cache.iter().map(|(metadata,_)| (metadata.timestamp.format("%y-%m-%d %H:%M:%S.%f").to_string(), metadata.id)).collect::<Vec<_>>();
            view!{
                <Section name = "Results">
                    <Panel>
                        <select name="sometext" size="5">
                            <For 
                                each = move ||result_summaries.clone().into_iter()
                                key = |summary|summary.clone()
                                let((timestamp, digitiser_id))
                            >
                                <option> "Timestamp: " {timestamp} ", Digitiser ID: " {digitiser_id} </option>
                            </For>
                        </select>
                    </Panel>
                    <Display />
                </Section>
            }.into_any()
        },
        Err(e) => view!{<h3> "Server Error:" {e.to_string()} </h3>}.into_any(),
    }
}