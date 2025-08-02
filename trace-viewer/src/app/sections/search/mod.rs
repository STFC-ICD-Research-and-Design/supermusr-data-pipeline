mod node_refs;
mod search_settings;
mod statusbar;

use leptos::{component, ev::SubmitEvent, prelude::*, view, IntoView};

use search_settings::SearchSettings;
use tracing::instrument;

use crate::{
    app::{sections::{search::node_refs::SearchBrokerNodeRefs, BrokerSettingsNodeRefs}, AppUuid}, structs::{SearchResults, SearchTarget, SearchTargetBy, SearchTargetMode}
};

#[server]
#[instrument(skip_all, err(level = "warn"))]
pub async fn search_broker(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    target: SearchTarget,
) -> Result<SearchResults, ServerFnError> {
    use crate::{
        DefaultData,
        finder::{MessageFinder, SearchEngine},
        structs::{SearchStatus, Topics},
        sessions::SessionEngine
    };
    use std::sync::{Arc, Mutex};
    use tracing::{debug, trace};

    debug!("search: {:?}", target);

    let session_engine = use_context::<Arc<Mutex<SessionEngine>>>()
        .expect("Session engine should be provided, this should never fail.");

    debug!("Session engine found.");

    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    debug!("default: {:?}", default);

    let consumer = supermusr_common::create_default_consumer(
        &broker,
        &default.username,
        &default.password,
        &consumer_group,
        None,
    )?;
    
    let key: String = SessionEngine::get_key().await?;

    debug!("Session Key Found: {:?}", target);

    let status = {
        let mut session_engine = session_engine.lock()
            .expect("Session engine should lock, this should never fail.");
        session_engine.query_session(&key, |session|session.status.clone())
    };

    debug!("Status found.");

    let mut searcher = SearchEngine::new(
        consumer,
        &Topics {
            trace_topic,
            digitiser_event_topic,
        },
        status
    );
    let search_result = searcher.search(target).await;

    debug!("Search Result Found.");
    trace!("SearchResult: {search_result:?}");
    
    let mut session_engine = session_engine.lock()
        .expect("Session engine should lock, this should never fail.");
    session_engine.modify_session(&key, |session|session.cache = search_result.cache.clone());

    debug!("Session updated.");

    Ok(search_result)
}

pub(crate) type SearchBrokerServerAction = ServerAction<SearchBroker>;

#[component]
pub(crate) fn Search(search_broker_action: SearchBrokerServerAction) -> impl IntoView {
    let broker_settings_node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");

    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

    let uuid = use_context::<AppUuid>();

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

        search_broker_action.dispatch(SearchBroker {
            broker: broker_settings_node_refs.get_broker(),
            trace_topic: broker_settings_node_refs.get_trace_topic(),
            digitiser_event_topic: broker_settings_node_refs.get_digitiser_event_topic(),
            consumer_group: broker_settings_node_refs.get_consumer_group(),
            target,
        });
    };

    view! {
        <form on:submit = on_submit>
            <SearchSettings />
        </form>
        //<Statusbar search_broker_action />
    }
}
