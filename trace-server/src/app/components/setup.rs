use chrono::Utc;
use leptos::{component, prelude::*, view, IntoView};

use crate::DefaultData;

use crate::app::components::{Panel, Section};

#[component]
pub(crate) fn Setup() -> impl IntoView {
    let (match_criteria, set_match_criteria) = signal(0);

    let default = use_context::<DefaultData>().unwrap_or_default();

    view! {
        <Section name = "Settings">

            <Panel name = "Search Type">
                <div class = "block">
                    <div class = "setup-control">
                        <label for = "search-by">
                            "Search By: "
                            <select id = "search-by">
                                <option value = "0">Timestamp</option>
                            </select>
                        </label>
                    </div>

                    <div class = "setup-control">
                        <label for = "date">
                            "Date: "
                            <input type = "date" id = "date" value = default.select.timestamp.unwrap_or_else(Utc::now).date_naive().to_string() />
                        </label>
                    </div>

                    <div class = "setup-control">
                        <label for = "time">
                            "Time: "
                            <input type = "time" id = "time" value = default.select.timestamp.unwrap_or_else(Utc::now).time().to_string() />
                        </label>
                    </div>
                </div>

                <div class = "block">
                    <div class = "setup-control">
                        <label for = "match-criteria">
                            "Match Criteria: "
                            <select id = "match-criteria" on:change:target =move |ev|set_match_criteria.set(ev.target().value().parse().expect(""))>
                                <option value = "0">"Channels"</option>
                                <option value = "1">"Digitiser IDs"</option>
                            </select>
                        </label>
                    </div>
                    
                    <Show when=move || match_criteria.get() == 0>
                        <div class = "setup-control">
                            <label for = "channels">
                                "Channels: "
                                <input type = "text" id = "channels" />
                            </label>
                        </div>
                    </Show>
                    
                    <Show when=move || match_criteria.get() == 1>
                        <div class="setup-control">
                            <label for = "digitiser-ids">"
                                Digitiser IDs: "
                                <input type = "text" id = "digitiser-ids" />
                            </label>
                        </div>
                    </Show>

                    <div class="setup-control">
                        <label for = "number">
                            "Number: "
                            <input type = "number" id = "number" />
                        </label>
                    </div>
                </div>
            </Panel>
        </Section>
    }
}

#[server]
pub async fn poll_broker() -> Result<(),ServerFnError> {
    Ok(())
}

#[component]
pub(crate) fn Controls() -> impl IntoView {
    view!{
        <Panel name = "Controls">
            <button class = "controls" value = "Poll Broker" />
            <button class = "controls" value = "Begin Search"/>
        </Panel>
    }
}