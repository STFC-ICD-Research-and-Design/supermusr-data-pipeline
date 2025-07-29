mod node_refs;
mod search_settings;

use leptos::{IntoView, component, ev::SubmitEvent, prelude::*, view};

use super::results::SearchResults;
use search_settings::SearchSettings;
use tracing::instrument;

use crate::{
    app::sections::{BrokerSettingsNodeRefs, search::node_refs::SearchBrokerNodeRefs},
    structs::{SearchResults, SearchTarget, SearchTargetBy, SearchTargetMode},
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
        structs::Topics,
    };
    use tracing::debug;

    debug!("search: {:?}", target);

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

    let mut searcher = SearchEngine::new(
        consumer,
        &Topics {
            trace_topic,
            digitiser_event_topic,
        },
    );
    let search_result = searcher.search(target).await;

    debug!("SearchResult: {search_result:?}");

    Ok(search_result)
}

pub(crate) type SearchBrokerServerAction = ServerAction<SearchBroker>;

#[component]
pub(crate) fn Search(search_broker_action: SearchBrokerServerAction) -> impl IntoView {
    let broker_settings_node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");

    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

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
    }
}
