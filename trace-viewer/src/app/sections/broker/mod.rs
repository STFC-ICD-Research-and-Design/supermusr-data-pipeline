mod broker_info;
mod broker_poller;

use leptos::{component, prelude::*, view, IntoView};
use broker_info::DisplayBrokerInfo;
use broker_poller::BrokerPoller;
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
    use crate::{DefaultData, structs::Topics, finder::{MessageFinder, SearchEngine}};
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

    debug!("Literally Finished {broker_info:?}");
    Ok(broker_info)
}

#[component]
pub(crate) fn Broker() -> impl IntoView {
    let poll_broker_action = ServerAction::<PollBroker>::new();
    view! {
        <Section name = "Broker" classes = vec!["broker"]>
            <BrokerPoller poll_broker_action/>
            <DisplayBrokerInfo poll_broker_action />
        </Section>
    }
}
