mod broker_info;
mod broker_setup;

use leptos::{component, prelude::*, view, IntoView};
use broker_info::DisplayBrokerInfo;
use broker_setup::BrokerSetup;
use crate::structs::BrokerInfo;
use crate::app::components::Section;
use tracing::instrument;

#[server]
#[instrument(skip_all)]
pub async fn poll_broker(
    broker: String,
    trace_topic: String,
    digitiser_event_topic: String,
    consumer_group: String,
    poll_broker_timeout_ms: u64
) -> Result<Option<BrokerInfo>,ServerFnError> {
    use crate::finder::{MessageFinder, SearchEngine};
    use tracing::debug;

    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    debug!("{default:?}");

    let consumer = supermusr_common::create_default_consumer(
        &broker,
        &default.username,
        &default.password,
        &consumer_group,
        None)?;

    let searcher = SearchEngine::new(
        consumer,
        &Topics{ trace_topic, digitiser_event_topic }
    );

    let broker_info = searcher
        .poll_broker(poll_broker_timeout_ms)
        .await;
    Ok(broker_info)
}

#[component]
pub(crate) fn Broker() -> impl IntoView {
/*
    let broker_node_ref = NodeRef::<Input>::new();
    let trace_topic_node_ref = NodeRef::<Input>::new();
    let events_topic_node_ref = NodeRef::<Input>::new();
    let consumer_group_node_ref = NodeRef::<Input>::new();
    let poll_broker_timeout_node_ref = NodeRef::<Input>::new();
    let package_topics_fn = move ||
        Option::zip(trace_topic_node_ref.get(), events_topic_node_ref.get())
            .map(|(trace_topic, digitiser_event_topic)|
                Topics {trace_topic: trace_topic.value(), digitiser_event_topic: digitiser_event_topic.value()}
            );

    let broker_info_poll_fn = move |_: &()| {
        poll_broker(PollBrokerData {
            broker: broker_node_ref.get().map(|broker|broker.value()),
            topics: package_topics_fn(),
        })
    };*/
    let poll_broker_action = ServerAction::<PollBroker>::new();

    view! {
        <Section name = "Broker">
            <BrokerSetup poll_broker_action />
            <DisplayBrokerInfo poll_broker_action />
        </Section>
    }
}
