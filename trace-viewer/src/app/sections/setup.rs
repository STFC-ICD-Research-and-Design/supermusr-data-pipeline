use chrono::Utc;
use leptos::{component, prelude::*, view, IntoView};

use crate::DefaultData;

use crate::app::components::{Panel, Section};

#[component]
pub(crate) fn Setup() -> impl IntoView {
    let (match_criteria, set_match_criteria) = signal(0);

    let default = use_context::<DefaultData>().unwrap_or_default();

    view! {
        <Section name = "Search">
            <Panel>
                <div class = "block">
                    <div class = "control-box">
                        <label for = "search-by">
                            "Search By: "
                        </label>
                        <select id = "search-by">
                            <option value = "0">Timestamp</option>
                        </select>
                    </div>

                    <div class = "control-box">
                        <label for = "date">
                            "Date: "
                        </label>
                        <input type = "date" id = "date" value = default.select.timestamp.unwrap_or_else(Utc::now).date_naive().to_string() />
                    </div>

                    <div class = "control-box">
                        <label for = "time">
                            "Time: "
                        </label>
                        <input type = "time" id = "time" value = default.select.timestamp.unwrap_or_else(Utc::now).time().to_string() />
                    </div>
                </div>

                <div class = "block">
                    <div class = "control-box">
                        <label for = "match-criteria">
                            "Match Criteria: "
                        </label>
                        <select id = "match-criteria" on:change:target =move |ev|set_match_criteria.set(ev.target().value().parse().expect(""))>
                            <option value = "0">"Channels"</option>
                            <option value = "1">"Digitiser IDs"</option>
                        </select>
                    </div>
                    
                    <Show when=move || match_criteria.get() == 0>
                        <div class = "control-box">
                            <label for = "channels">
                                "Channels: "
                            </label>
                            <input type = "text" id = "channels" />
                        </div>
                    </Show>
                    
                    <Show when=move || match_criteria.get() == 1>
                        <div class="control-box">
                            <label for = "digitiser-ids">"
                                Digitiser IDs: "
                            </label>
                            <input type = "text" id = "digitiser-ids" />
                        </div>
                    </Show>

                    <div class="control-box">
                        <label for = "number">
                            "Number: "
                        </label>
                        <input type = "number" id = "number" />
                    </div>
                </div>
            </Panel>

            <Panel>
                <button type = "submit">Search</button>
                //<Progress />
                //<Status />
            </Panel>
        </Section>
    }
}