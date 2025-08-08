use chrono::Utc;
use leptos::html::Input;
use leptos::{IntoView, component, prelude::*, view};
use strum::IntoEnumIterator;

use crate::DefaultData;
use crate::structs::{SearchBy, SearchMode, Select};

use crate::app::components::InputBoxWithLabel;

use super::node_refs::SearchBrokerNodeRefs;

#[component]
pub(crate) fn SearchSettings() -> impl IntoView {
    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    let default = use_context::<DefaultData>()
        .expect("Default Data should be provided, this should never fail.");

    let default_timestamp = default.select.timestamp.unwrap_or_else(Utc::now);
    let default_date = default_timestamp.date_naive().to_string();
    let default_time = default_timestamp.time().to_string();
    let default_number = default.select.number.unwrap_or(1).to_string();

    let search_broker_node_refs = use_context::<SearchBrokerNodeRefs>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    view! {
        <SearchMode />
        <InputBoxWithLabel name = "date" label = "Date:" input_type = "date" value = default_date node_ref = search_broker_node_refs.date_ref />
        <InputBoxWithLabel name = "time" label = "Time:" input_type = "text" value = default_time node_ref = search_broker_node_refs.time_ref />

        <MatchCriteria match_criteria set_match_criteria/>
        <MatchBy match_criteria select = default.select channels_ref = search_broker_node_refs.channels_ref digitiser_ids_ref = search_broker_node_refs.digitiser_ids_ref />
        <InputBoxWithLabel name = "number" label = "Number:" input_type = "number" value = default_number node_ref = search_broker_node_refs.number_ref />
    }
}

#[component]
pub(crate) fn SearchMode() -> impl IntoView {
    let (search_mode, set_search_mode) = signal(SearchMode::Timestamp);

    let search_broker_node_refs = use_context::<SearchBrokerNodeRefs>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    view! {
        <label class = "panel-item" for = "search-mode">
            "Search Mode: "
            <select name = "search-mode" id = "search-mode" class = "panel-item" node_ref = search_broker_node_refs.search_mode_ref on:change:target = move |ev|
                set_search_mode.set(
                    ev.target()
                        .value()
                        .parse()
                        .expect("SearchMode value should parse, this should never fail.")
                    ) >
                <For
                    each = SearchMode::iter
                    key = |mode|mode.to_string()
                    let(mode)
                >
                    <option selected={search_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                </For>
            </select>
        </label>
    }
}

#[component]
pub(crate) fn MatchCriteria(
    match_criteria: ReadSignal<SearchBy>,
    set_match_criteria: WriteSignal<SearchBy>,
) -> impl IntoView {
    let search_broker_node_refs = use_context::<SearchBrokerNodeRefs>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    view! {
        <label for = "match-criteria">
            "Match Criteria: "
            <select name = "match-criteria" id = "match-criteria" class = "panel-item" node_ref = search_broker_node_refs.search_by_ref on:change:target = move |ev| set_match_criteria.set(ev.target().value().parse().expect("SearchBy value should parse, this should never fail.")) >
                <For
                    each = SearchBy::iter
                    key = |mode|mode.to_string()
                    let(mode)
                >
                    <option selected={match_criteria.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                </For>
            </select>
        </label>
    }
}

#[component]
pub(crate) fn MatchBy(
    match_criteria: ReadSignal<SearchBy>,
    select: Select,
    channels_ref: NodeRef<Input>,
    digitiser_ids_ref: NodeRef<Input>,
) -> impl IntoView {
    fn parse_to_list<T: ToString>(list: Option<&[T]>) -> String {
        list.unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    move || match match_criteria.get() {
        SearchBy::ByChannels => {
            let channels = parse_to_list(select.channels.as_deref());
            view! {
                <label for = "channels">
                    "Channels:"
                    <input class = "panel-item" type = "text" id = "channels" value = channels node_ref = channels_ref />
                </label>
            }
        }
        SearchBy::ByDigitiserIds => {
            let digitiser_ids = parse_to_list(select.digitiser_ids.as_deref());
            view! {
                <label for = "digitiser-ids">
                    "Digitiser IDs:"
                    <input class = "panel-item" type = "text" id = "digitiser-ids" value = digitiser_ids node_ref = digitiser_ids_ref/>
                </label>
            }
        }
    }
}
