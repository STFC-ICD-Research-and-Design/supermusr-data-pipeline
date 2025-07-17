use chrono::Utc;
use leptos::{component, prelude::*, view, IntoView};
use strum::IntoEnumIterator;

use crate::structs::{SearchBy, SearchMode, SearchTargetMode};
use crate::DefaultData;

use crate::app::components::{ControlBox, ControlBoxWithLabel, Panel, Section, VerticalBlock};

#[component]
pub(crate) fn Search(default: DefaultData) -> impl IntoView {
    let (search_mode, set_search_mode) = signal(SearchMode::Timestamp);
    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    //let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    view! {
        <Section name = "Search">
            <Panel>
                <VerticalBlock>
                    <ControlBoxWithLabel name = "search-mode" label = "Search Mode: ">
                        <select id = "search-mode" on:change:target =move |ev|set_search_mode.set(ev.target().value().parse().expect("SearchMode value should parse, this should never fail."))>
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
                        <input type = "date" id = "date" value = default.select.timestamp.unwrap_or_else(Utc::now).date_naive().to_string() />
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "time" label = "Time: ">
                        <input type = "time" id = "time" value = default.select.timestamp.unwrap_or_else(Utc::now).time().to_string() />
                    </ControlBoxWithLabel>
                </VerticalBlock>

                <VerticalBlock>
                    <ControlBoxWithLabel name = "match-criteria" label = "Match Criteria: ">
                        <select id = "match-criteria" on:change:target =move |ev|set_match_criteria.set(ev.target().value().parse().expect("SearchBy value should parse, this should never fail."))>
                            <For
                                each = SearchBy::iter
                                key = |mode|mode.to_string()
                                let(mode)
                            >
                                <option selected={match_criteria.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                            </For>
                        </select>
                    </ControlBoxWithLabel>
                    
                    {move ||
                        match match_criteria.get() {
                            SearchBy::ByChannels => view!{
                                <ControlBoxWithLabel name = "channels" label = "Channels: ">
                                    <input type = "text" id = "channels" />
                                </ControlBoxWithLabel>
                            },
                            SearchBy::ByDigitiserIds => view!{
                                <ControlBoxWithLabel name = "digitiser-ids" label = "Digitiser IDs: ">
                                    <input type = "text" id = "digitiser-ids" />
                                </ControlBoxWithLabel>
                            },
                        }
                    }

                    <ControlBoxWithLabel name = "number" label = "Number: ">
                        <input type = "number" id = "number" />
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