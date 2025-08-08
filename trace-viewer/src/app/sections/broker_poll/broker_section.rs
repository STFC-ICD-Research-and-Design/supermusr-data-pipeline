use super::{broker_info::DisplayBrokerInfo, broker_poller::BrokerPoller};
use crate::app::{components::Section, server_functions::PollBroker};
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn BrokerSection() -> impl IntoView {
    let poll_broker_action = ServerAction::<PollBroker>::new();
    view! {
        <Section text = "Broker" id = "broker">
            <BrokerPoller poll_broker_action/>
            <DisplayBrokerInfo poll_broker_action />
        </Section>
    }
}
