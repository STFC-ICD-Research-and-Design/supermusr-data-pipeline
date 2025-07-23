use chrono::Utc;
use leptos::html::Input;
use leptos::{component, prelude::*, view, IntoView};
use strum::IntoEnumIterator;

use crate::app::sections::statusbar::Status;
use crate::structs::{SearchBy, SearchMode, Select};
use crate::DefaultData;

use crate::app::components::{ControlBoxWithLabel, InputBoxWithLabel, Panel, Section, SubmitBox};

#[component]
pub(crate) fn SearchSettings(date_ref: NodeRef<Input>, time_ref: NodeRef<Input>,  number_ref: NodeRef<Input>,  channels_ref: NodeRef<Input>, digitiser_ids_ref: NodeRef<Input>) -> impl IntoView {
    let (search_mode, set_search_mode) = signal(SearchMode::Timestamp);
    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    let default_date = default.select.timestamp.unwrap_or_else(Utc::now).date_naive().to_string();
    let default_time = default.select.timestamp.unwrap_or_else(Utc::now).time().to_string();
    let default_number = default.select.number.unwrap_or(1).to_string();

    view! {
        <Section name = "Search" classes = vec!["search-setup"]>
            <Panel classes = vec!["search-setup"]>
                <ControlBoxWithLabel name = "search-mode" label = "Search Mode: ">
                    <select name = "search-mode panel-item" on:change:target = move |ev| set_search_mode.set(ev.target().value().parse().expect("SearchMode value should parse, this should never fail.")) >
                        <For
                            each = SearchMode::iter
                            key = |mode|mode.to_string()
                            let(mode)
                        >
                            <option selected={search_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                        </For>
                    </select>
                </ControlBoxWithLabel>

                <InputBoxWithLabel name = "date" label = "Date:" input_type = "date" value = default_date node_ref = date_ref />
                <InputBoxWithLabel name = "time" label = "Time:" input_type = "text" value = default_time node_ref = time_ref />
            </Panel>

            <Panel classes = vec!["search-setup"]>
                <ControlBoxWithLabel name = "match-criteria" label = "Match Criteria: ">
                    <select id = "match-criteria panel-item" on:change:target = move |ev| set_match_criteria.set(ev.target().value().parse().expect("SearchBy value should parse, this should never fail.")) >
                        <For
                            each = SearchBy::iter
                            key = |mode|mode.to_string()
                            let(mode)
                        >
                            <option selected={match_criteria.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                        </For>
                    </select>
                </ControlBoxWithLabel>

                <MatchBy match_criteria select = default.select channels_ref digitiser_ids_ref />
                <InputBoxWithLabel name = "number" label = "Number:" input_type = "number" value = default_number node_ref = number_ref />
            </Panel>
            <SubmitBox label = "Search" />
            //<Progress />
            //<Status />
        </Section>
    }
}

#[component]
pub(crate) fn MatchBy(match_criteria: ReadSignal<SearchBy>, select: Select, channels_ref: NodeRef<Input>, digitiser_ids_ref: NodeRef<Input>) -> impl IntoView {
    move ||match match_criteria.get() {
        SearchBy::ByChannels => {
            let channels = select
                .channels
                .clone()
                .unwrap_or_default()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");

            view! {
                <ControlBoxWithLabel name = "channels" label = "Channels:">
                    <input class = "panel-item" type = "text" id = "channels" value = channels node_ref = channels_ref />
                </ControlBoxWithLabel>
            }
        },
        SearchBy::ByDigitiserIds => {
            let digitiser_ids = select
                .digitiser_ids
                .clone()
                .unwrap_or_default()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");

            view! {
                <ControlBoxWithLabel name = "digitiser-ids" label = "Digitiser IDs:">
                    <input class = "panel-item" type = "text" id = "digitiser-ids" value = digitiser_ids node_ref = digitiser_ids_ref/>
                </ControlBoxWithLabel>
            }
        },
    }
}