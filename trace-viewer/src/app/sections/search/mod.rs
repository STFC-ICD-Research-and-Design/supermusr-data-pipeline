mod node_refs;
mod search_settings;

use leptos::{IntoView, component, ev::SubmitEvent, prelude::*, view};

use search_settings::SearchSettings;
use tracing::{warn};

use crate::{
    app::{sections::{search::node_refs::SearchBrokerNodeRefs, BrokerSettingsNodeRefs}, server_functions::{CreateNewSearch, GetSearchResults}},
    structs::{SearchTarget, SearchTargetBy, SearchTargetMode},
};

#[component]
pub(crate) fn Search() -> impl IntoView {
    let broker_settings_node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");

    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

    let get_search_results_action = use_context::<ServerAction<GetSearchResults>>().expect("");
    let create_new_search_action = use_context::<ServerAction<CreateNewSearch>>().expect("");

    let on_submit = move |e: SubmitEvent| {
        e.prevent_default();
        let time = search_broker_node_refs.get_time();
        let date = search_broker_node_refs.get_date();
        let timestamp = date.and_time(time).and_utc();

        let channels = search_broker_node_refs.get_channels();
        let number = search_broker_node_refs.get_number();

        let target = SearchTarget {
            mode: SearchTargetMode::Timestamp { timestamp },
            by: SearchTargetBy::ByChannels { channels },
            number,
        };

        create_new_search_action.dispatch(CreateNewSearch {
            broker: broker_settings_node_refs.get_broker(),
            trace_topic: broker_settings_node_refs.get_trace_topic(),
            digitiser_event_topic: broker_settings_node_refs.get_digitiser_event_topic(),
            consumer_group: broker_settings_node_refs.get_consumer_group(),
            target,
        });
    };

    Effect::new(move || {
        if let Some(uuid) = create_new_search_action.value().get() {
            match uuid {
                Ok(uuid) => {
                    get_search_results_action.dispatch(GetSearchResults { uuid });
                }
                Err(e) => warn!("{e}"),
            }
        }
    });

    view! {
        <form on:submit = on_submit>
            <SearchSettings />
        </form>
        //<Statusbar search_broker_action />
    }
}
