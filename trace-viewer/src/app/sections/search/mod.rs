mod node_refs;
mod search_settings;
mod statusbar;

use leptos::{IntoView, component, ev::SubmitEvent, prelude::*, view};

use search_settings::SearchSettings;
use tracing::{instrument, warn};

use crate::{
    app::sections::{BrokerSettingsNodeRefs, search::node_refs::SearchBrokerNodeRefs},
    structs::{SearchResults, SearchTarget, SearchTargetBy, SearchTargetMode},
};

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn create_new_search(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    target: SearchTarget,
) -> Result<String, ServerFnError> {
    use crate::{
        DefaultData,
        finder::{MessageFinder, SearchEngine},
        sessions::SessionEngine,
        structs::{SearchStatus, Topics},
    };
    use std::sync::{Arc, Mutex};
    use tracing::{debug, trace};

    debug!("Creating new search task for target: {:?}", target);

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");

    let uuid = session_engine.create_new_search(
        broker,
        trace_topic,
        digitiser_event_topic,
        consumer_group,
        target,
    )?;

    debug!("New search task has uuid: {}", uuid);

    Ok(uuid)
}

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn get_search_results(uuid: String) -> Result<SearchResults, ServerFnError> {
    use crate::sessions::SessionEngine;
    use std::sync::{Arc, Mutex};

    let session_engine_arc_mutex = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    let mut session_engine = session_engine_arc_mutex.lock().expect("");
    Ok(session_engine.get_search_results(uuid).await)
}

pub(crate) type GetSearchResultsServerAction = ServerAction<GetSearchResults>;

#[component]
pub(crate) fn Search(get_search_results_action: GetSearchResultsServerAction) -> impl IntoView {
    let broker_settings_node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");

    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

    let create_new_search_action = ServerAction::<CreateNewSearch>::new();

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
