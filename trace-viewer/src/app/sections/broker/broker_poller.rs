use crate::{app::components::{SubmitBox, InputBoxWithLabel, Panel}, DefaultData};
use super::PollBroker;
use leptos::{component, prelude::*, view, IntoView, ServerAction};

#[component]
pub fn BrokerPoller(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let node_refs = use_context::<BrokerSettingsNodeRefs>().expect("Node refs should be available, this should never fail.");
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");
    let submit_action = move |_| {
        node_refs.broker.get();
        poll_broker_action.dispatch();
    };
    view! {
        <Panel classes = vec!["broker-setup"]>
            <form on:submit = submit_action>
                <InputBoxWithLabel name = "poll_broker_timeout_ms" label = "Poll Broker Timeout (ms):" input_type = "number" value = default.poll_broker_timeout_ms />

                <SubmitBox label = "Poll Broker" classes = vec!["across-two-cols"]/>
            </form>
        </Panel>
    }
}