use crate::{app::components::{SubmitBox, InputBoxWithLabel, Panel}, DefaultData};
use leptos::{component, prelude::*, view, IntoView};

#[component]
pub fn BrokerSetup() -> impl IntoView {
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");
    view! {
        <Panel classes = vec!["broker-setup"]>
            <InputBoxWithLabel name = "broker" label = "Broker URI: " input_type = "text" value = default.broker />
            <InputBoxWithLabel name = "trace_topic" label = "Trace Topic:" input_type = "text" value = default.topics.trace_topic />
            <InputBoxWithLabel name = "digitiser_event_topic" label = "Eventlist Topic:" input_type = "text" value = default.topics.digitiser_event_topic />
            <InputBoxWithLabel name = "consumer_group" label = "Consumer Group:" input_type = "text" value = default.consumer_group />
            <InputBoxWithLabel name = "poll_broker_timeout_ms" label = "Poll Broker Timeout (ms):" input_type = "number" value = default.poll_broker_timeout_ms />

            <SubmitBox label = "Poll Broker" classes = vec!["across-two-cols"]/>
        </Panel>
    }
}