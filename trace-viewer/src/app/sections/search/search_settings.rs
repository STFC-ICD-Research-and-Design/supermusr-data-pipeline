use crate::app::{TopLevelContext, sections::search::search_section::SearchLevelContext};
use chrono::Utc;
use leptos::{IntoView, component, prelude::*, view};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[component]
pub(crate) fn SearchSettings() -> impl IntoView {
    let default_data = use_context::<TopLevelContext>()
        .expect("TopLevelContext should be provided, this should never fail.")
        .client_side_data
        .default_data;

    let search_broker_node_refs = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.")
        .search_broker_node_refs;

    let default_timestamp = default_data.timestamp.unwrap_or_else(Utc::now);
    let default_date = default_timestamp.date_naive().to_string();
    let default_time = default_timestamp.time().to_string();
    let default_number = default_data.number.unwrap_or(1).to_string();

    let (match_criteria, set_match_criteria) = signal(SearchBy::ByChannels);

    view! {
        <SearchMode />
        <label for = "date">
            "Date:"
            <input name = "date" id = "date" value = default_date type = "date" node_ref = search_broker_node_refs.date_ref />
        </label>
        <label for = "time">
            "Time:"
            <input name = "time" id = "time" value = default_time type = "text" node_ref = search_broker_node_refs.time_ref />
        </label>

        <MatchCriteria match_criteria set_match_criteria/>
        <MatchBy match_criteria />
        <label for = "number">
            "Number:"
            <input name = "number" id = "number" value = default_number type = "text" node_ref = search_broker_node_refs.number_ref />
        </label>
    }
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub(crate) enum SearchMode {
    #[default]
    #[strum(to_string = "From Timestamp")]
    Timestamp,
}

#[component]
pub(crate) fn SearchMode() -> impl IntoView {
    let (search_mode, set_search_mode) = signal(SearchMode::Timestamp);

    let search_broker_node_refs = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.")
        .search_broker_node_refs;

    view! {
        <label class = "panel-item" for = "search-mode">
            "Search Mode: "
            <select name = "search-mode" id = "search-mode" class = "panel-item" node_ref = search_broker_node_refs.search_mode_ref
                on:change:target = move |ev|
                    set_search_mode.set(
                        ev.target()
                            .value()
                            .parse()
                            .expect("SearchMode value should parse, this should never fail.")
                        ) >
                <For each = SearchMode::iter
                    key = ToOwned::to_owned
                    let(mode)
                >
                    <option selected={search_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                </For>
            </select>
        </label>
    }
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub(crate) enum SearchBy {
    #[default]
    #[strum(to_string = "By Channels")]
    ByChannels,
    #[strum(to_string = "By Digitiser Ids")]
    ByDigitiserIds,
}

#[component]
pub(crate) fn MatchCriteria(
    match_criteria: ReadSignal<SearchBy>,
    set_match_criteria: WriteSignal<SearchBy>,
) -> impl IntoView {
    let search_broker_node_refs = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.")
        .search_broker_node_refs;

    view! {
        <label for = "match-criteria">
            "Match Criteria: "
            <select name = "match-criteria" id = "match-criteria" class = "panel-item"
                data-tooltip = "Choose which criteria to match on: by channel's contained, or by digitiser id."
                node_ref = search_broker_node_refs.search_by_ref
                on:change:target = move |ev| set_match_criteria.set(
                    ev.target().value()
                        .parse()
                        .expect("SearchBy value should parse, this should never fail.")
                )
            >
                <For
                    each = SearchBy::iter
                    key = ToOwned::to_owned
                    let(mode)
                >
                    <option selected={match_criteria.get() == mode}  value = {mode.to_string()}>{mode.to_string()}</option>
                </For>
            </select>
        </label>
    }
}

#[component]
pub(crate) fn MatchBy(match_criteria: ReadSignal<SearchBy>) -> impl IntoView {
    let default_data = use_context::<TopLevelContext>()
        .expect("TopLevelContext should be provided, this should never fail.")
        .client_side_data
        .default_data;

    let search_broker_node_refs = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.")
        .search_broker_node_refs;

    fn parse_to_list<T: ToString>(list: Option<&[T]>) -> String {
        list.unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }

    move || match match_criteria.get() {
        SearchBy::ByChannels => {
            let channels = parse_to_list(default_data.channels.as_deref());
            view! {
                <label for = "channels">
                    "Channels:"
                    <input class = "panel-item" type = "text" id = "channels" value = channels node_ref = search_broker_node_refs.channels_ref />
                </label>
            }
        }
        SearchBy::ByDigitiserIds => {
            let digitiser_ids = parse_to_list(default_data.digitiser_ids.as_deref());
            view! {
                <label for = "digitiser-ids">
                    "Digitiser IDs:"
                    <input class = "panel-item" type = "text" id = "digitiser-ids" value = digitiser_ids node_ref = search_broker_node_refs.digitiser_ids_ref/>
                </label>
            }
        }
    }
}
