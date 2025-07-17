use chrono::Utc;
use leptos::{component, prelude::*, view, IntoView};

use crate::structs::SearchBy;
use crate::DefaultData;

use crate::app::components::{ControlBox, ControlBoxWithLabel, Panel, Section, VerticalBlock};

#[component]
pub(crate) fn Search() -> impl IntoView {
    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    let default = use_context::<DefaultData>().unwrap_or_default();

    view! {
        <Section name = "Search">
            <Panel>
                <VerticalBlock>
                    <ControlBoxWithLabel name = "search-by" label = "Search By: ">
                        <select id = "search-by">
                            <option value = "0">Timestamp</option>
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
                        <select id = "match-criteria" on:change:target =move |ev|set_match_criteria.set(ev.target().value().parse().expect(""))>
                            <option value = "0">"Channels"</option>
                            <option value = "1">"Digitiser IDs"</option>
                        </select>
                    </ControlBoxWithLabel>
                    
                    {
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
                <ControlBox>
                    <button type = "submit">Search</button>
                </ControlBox>
                //<Progress />
                //<Status />
            </Panel>
        </Section>
    }
}