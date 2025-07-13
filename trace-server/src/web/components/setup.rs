use leptos::{component, prelude::*, task::spawn_local, view, IntoView};

use crate::{finder::MessageFinder, web::components::{Panel, Section}};

#[component]
pub(crate) fn Setup() -> impl IntoView {
    let (match_criteria, set_match_criteria) = signal(0);
    view! {
        <Section name = "Settings">
            <Panel name = "Broker">
                <div class = "block">
                    <div class = "setup-control">
                        <label for = "broker">"Broker URI: "</label>
                        <input type = "url" id = "broker"/>
                    </div>

                    <div class = "setup-control">
                        <label for = "trace-topic">"Trace Topic: "</label>
                        <input type = "text" id = "trace-topic"></input>
                    </div>

                    <div class = "setup-control">
                        <label for = "eventlist-topic">"Eventlist Topic: "</label>
                        <input type = "text" id = "eventlist-topic"></input>
                    </div>
                </div>
            </Panel>

            <Panel name = "Search Type">
                <div class = "block">
                    <div class = "setup-control">
                        <label for = "search-by">"Search By: "</label>
                        <select id = "search-by">
                            <option value = "0">Timestamp</option>
                        </select>
                    </div>

                    <div class = "setup-control">
                        <label for = "date">"Date: "</label>
                        <input type = "date" id = "date"></input>
                    </div>

                    <div class = "setup-control">
                        <label for = "time">"Time: "</label>
                        <input type = "time" id = "time"></input>
                    </div>
                </div>

                <div class = "block">
                    <div class = "setup-control">
                        <label for = "match-criteria">"Match Criteria: "</label>
                        <select id = "match-criteria" on:change:target =move |ev|set_match_criteria.set(ev.target().value().parse().expect(""))>
                            <option value = "0">"Channels"</option>
                            <option value = "1">"Digitiser IDs"</option>
                        </select>
                    </div>
                    
                    <Show when=move || match_criteria.get() == 0>
                        <div class = "setup-control">
                            <label for = "channels">"Channels: "</label>
                            <input type = "text" id = "channels"></input>
                        </div>
                    </Show>
                    
                    <Show when=move || match_criteria.get() == 1>
                        <div class="setup-control">
                            <label for = "digitiser-ids">"Digitiser IDs: "</label>
                            <input type = "text" id = "digitiser-ids"></input>
                        </div>
                    </Show>

                    <div class="setup-control">
                        <label for = "number">"Number: "</label>
                        <input type = "number" id = "number"></input>
                    </div>
                </div>
            </Panel>
        </Section>
    }
}

pub async fn poll_broker<Finder: MessageFinder>(finder : Finder) {
}

#[component]
pub(crate) fn Controls<Finder: MessageFinder>(finder : Finder) -> impl IntoView {
    view!{
        <Panel name = "Controls">
            <button class = "controls" value = "Poll Broker" />
            <button class = "controls" value = "Begin Search"/>
        </Panel>
    }
}