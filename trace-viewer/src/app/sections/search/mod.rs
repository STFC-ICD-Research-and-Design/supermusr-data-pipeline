mod results;
mod search_settings;

use chrono::{NaiveDate, NaiveTime};
use leptos::{component, html::Input, prelude::*, view, IntoView, ev::SubmitEvent};

use search_settings::SearchSettings;
use results::SearchResults;
use tracing::{debug, instrument};

use crate::{app::sections::BrokerSettingsNodeRefs, structs::{SearchResults, SearchStatus, SearchTarget, SearchTargetBy, SearchTargetMode, Topics}};

#[server]
#[instrument(skip_all, err(level = "warn"))]
async fn search_broker(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    target: SearchTarget
) -> Result<SearchResults,ServerFnError> {
    use crate::{DefaultData, finder::{MessageFinder, SearchEngine}};
    
    debug!("search: {:?}", target);
    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    debug!("default: {:?}", default);

    let consumer = supermusr_common::create_default_consumer(
        &broker,
        &default.username,
        &default.password,
        &consumer_group,
        None)?;

    let mut searcher = SearchEngine::new(
        consumer,
        &Topics { trace_topic, digitiser_event_topic }
    );
    let search_result = searcher.search(target)
        .await;
    
    debug!("SearchResult: {search_result:?}");

    Ok(search_result)
}

#[derive(Default, Clone, Copy)]
struct SearchBrokerNodeRefs {
    date_ref: NodeRef<Input>,
    time_ref: NodeRef<Input>,
    number_ref: NodeRef<Input>,
    channels_ref: NodeRef<Input>,
    digitiser_ids_ref: NodeRef<Input>,
}

impl SearchBrokerNodeRefs {
    fn get_time(&self) -> NaiveTime {
        self.time_ref.get()
            .expect("time ref Should exists, this should never fail.")
            .value()
            .parse::<NaiveTime>()
            .expect("time should be NaiveTime, this should never fail.")
    }

    fn get_date(&self) -> NaiveDate {
        self.date_ref.get()
            .expect("date ref Should exists, this should never fail.")
            .value()
            .parse::<NaiveDate>()
            .expect("date should be NaiveDate, this should never fail.")
    }

    fn get_channels(&self) -> Vec<u32> {
        self.channels_ref.get()
            .expect("channels ref should exist, this should never fail.")
            .value()
            .split(",")
            .map(|x|x.parse())
            .collect::<Result<Vec<_>,_>>()
            .expect("")
    }

    fn get_number(&self) -> usize {
        self.number_ref.get()
            .expect("number ref should exist, this should never fail.")
            .value()
            .parse()
            .unwrap_or(1)
    }
}

#[component]
pub(crate) fn Search() -> impl IntoView {
    let broker_settings_node_refs = use_context::<BrokerSettingsNodeRefs>().expect("Node refs should be available, this should never fail.");
    
    //let search_mode_ref = NodeRef::<Input>::new();
    //let search_by_ref = NodeRef::<Input>::new();
    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

    let search_broker_action = ServerAction::<SearchBroker>::new();

    let on_submit = move |e : SubmitEvent| {
        e.prevent_default();
        let time = search_broker_node_refs.get_time();
        let date = search_broker_node_refs.get_date();
        let timestamp = date.and_time(time).and_utc();

        let channels = search_broker_node_refs.get_channels();
        let number = search_broker_node_refs.get_number();

        let target = SearchTarget {
            mode: SearchTargetMode::Timestamp { timestamp },
            by: SearchTargetBy::ByChannels { channels },
            number
        };

        search_broker_action.dispatch(SearchBroker { 
            broker: broker_settings_node_refs.broker.get()
                .expect("broker node_ref should exist, this should never fail.")
                .value(),
            trace_topic: broker_settings_node_refs.trace_topic.get()
                .expect("trace_topic node_ref should exist, this should never fail.")
                .value(),
            digitiser_event_topic: broker_settings_node_refs.digitiser_event_topic.get()
                .expect("digitiser_event_topic node_ref should exist, this should never fail.")
                .value(),
            consumer_group: broker_settings_node_refs.consumer_group.get()
                .expect("consumer_group node_ref should exist, this should never fail.")
                .value(),
            target
        });
    };

    view! {
        <form on:submit = on_submit>
            <SearchSettings />
            <SearchResults search_broker_action />
        </form>
    }
}