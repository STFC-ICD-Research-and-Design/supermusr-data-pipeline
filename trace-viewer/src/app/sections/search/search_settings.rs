use chrono::Utc;
use leptos::ev::Targeted;
use leptos::html::Input;
use leptos::{component, prelude::*, view, IntoView};
use strum::IntoEnumIterator;

use crate::structs::{SearchBy, SearchMode, SearchTargetMode};
use crate::DefaultData;

use crate::app::components::{ControlBox, ControlBoxWithLabel, Panel, Section, VerticalBlock};

#[component]
pub(crate) fn SearchSettings(date_ref: NodeRef<Input>, time_ref: NodeRef<Input>,  number_ref: NodeRef<Input>,  channels_ref: NodeRef<Input>) -> impl IntoView {
    let (search_mode, set_search_mode) = signal(SearchMode::Timestamp);
    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    view! {
        <Section name = "Search">
            <Panel>
                <VerticalBlock>
                    <ControlBoxWithLabel name = "search-mode" label = "Search Mode: ">
                        <select id = "search-mode" on:change:target = move |ev| set_search_mode.set(ev.target().value().parse().expect("SearchMode value should parse, this should never fail.")) >
                            <For
                                each = SearchMode::iter
                                key = |mode|mode.to_string()
                                let(mode)
                            >
                                <option selected={search_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                            </For>
                        </select>
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "date" label = "Date: ">
                        <input type = "date" id = "date" value = default.select.timestamp.unwrap_or_else(Utc::now).date_naive().to_string() node_ref = date_ref />
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "time" label = "Time: ">
                        <input type = "text" id = "time" value = default.select.timestamp.unwrap_or_else(Utc::now).time().to_string() node_ref = time_ref />
                    </ControlBoxWithLabel>
                </VerticalBlock>

                <VerticalBlock>
                    <ControlBoxWithLabel name = "match-criteria" label = "Match Criteria: ">
                        <select id = "match-criteria" on:change:target = move |ev| set_match_criteria.set(ev.target().value().parse().expect("SearchBy value should parse, this should never fail.")) >
                            <For
                                each = SearchBy::iter
                                key = |mode|mode.to_string()
                                let(mode)
                            >
                                <option selected={match_criteria.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                            </For>
                        </select>
                    </ControlBoxWithLabel>
                    
                    {
                        move || {
                            let channels = default.select
                                .channels
                                .clone()
                                .unwrap_or_default()
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(",");

                            let digitiser_ids = default.select
                                .digitiser_ids
                                .clone()
                                .unwrap_or_default()
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(",");

                            match match_criteria.get() {
                                SearchBy::ByChannels => {
                                    view! {
                                        <ControlBoxWithLabel name = "channels" label = "Channels: ">
                                            <input type = "text" id = "channels" value = channels node_ref = channels_ref />
                                        </ControlBoxWithLabel>
                                    }
                                },
                                SearchBy::ByDigitiserIds => {;
                                    view!{
                                        <ControlBoxWithLabel name = "digitiser-ids" label = "Digitiser IDs: ">
                                            <input type = "text" id = "digitiser-ids" value = digitiser_ids />
                                        </ControlBoxWithLabel>
                                    }
                                },
                            }
                        }
                    }

                    <ControlBoxWithLabel name = "number" label = "Number: ">
                        <input type = "number" id = "number" node_ref = number_ref />
                    </ControlBoxWithLabel>
                </VerticalBlock>
            </Panel>

            <Panel>
                <VerticalBlock>
                    <ControlBox>
                        <input type = "submit" value = "Search" />
                    </ControlBox>
                    //<Progress />
                    //<Status />
                </VerticalBlock>
            </Panel>
        </Section>
    }
}