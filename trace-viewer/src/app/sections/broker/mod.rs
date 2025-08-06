mod broker_info;
mod broker_poller;

use crate::app::{components::Section, server_functions::PollBroker};
use broker_info::DisplayBrokerInfo;
use broker_poller::BrokerPoller;
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn Broker() -> impl IntoView {
    let poll_broker_action = ServerAction::<PollBroker>::new();
    view! {
        <Section name = "Broker" classes = vec!["broker"] closable = true>
            <BrokerPoller poll_broker_action/>
            <DisplayBrokerInfo poll_broker_action />
        </Section>
    }
}
