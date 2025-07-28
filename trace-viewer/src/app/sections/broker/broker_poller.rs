use crate::{app::components::{SubmitBox, InputBoxWithLabel, Panel}, DefaultData};
use super::{PollBroker, super::BrokerSettingsNodeRefs};
use leptos::{component, ev::SubmitEvent, html::Input, prelude::*, view, IntoView};

#[component]
pub fn BrokerPoller(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let node_refs = use_context::<BrokerSettingsNodeRefs>().expect("Node refs should be available, this should never fail.");
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    let timeout_ms_ref = NodeRef::<Input>::new();

    let submit_action = move |e : SubmitEvent| {
        e.prevent_default();
        poll_broker_action.dispatch(PollBroker {
            broker: node_refs.broker.get()
                .expect("broker node_ref should exist, this should never fail.")
                .value(),
            trace_topic: node_refs.trace_topic.get()
                .expect("trace_topic node_ref should exist, this should never fail.")
                .value(),
            digitiser_event_topic: node_refs.digitiser_event_topic.get()
                .expect("digitiser_event_topic node_ref should exist, this should never fail.")
                .value(),
            consumer_group: node_refs.consumer_group.get()
                .expect("consumer_group node_ref should exist, this should never fail.")
                .value(),
            poll_broker_timeout_ms: timeout_ms_ref.get()
                .expect("poll_broker_timeout node_ref should exist, this should never fail.")
                .value()
                .parse()
                .expect("poll_broker_timeout string should be u64, this should never fail."),
        });
    };
    view! {
        <Panel classes = vec!["broker-setup"]>
            <form on:submit = submit_action>
                <InputBoxWithLabel name = "poll_broker_timeout_ms" label = "Poll Broker Timeout (ms):" input_type = "number" value = default.poll_broker_timeout_ms node_ref = timeout_ms_ref />

                <SubmitBox label = "Poll Broker" classes = vec!["across-two-cols"]/>
            </form>
        </Panel>
    }
}