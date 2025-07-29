use super::{super::BrokerSettingsNodeRefs, PollBroker};
use crate::{
    DefaultData,
    app::components::{InputBoxWithLabel, Panel, SubmitBox},
};
use leptos::{IntoView, component, ev::SubmitEvent, html::Input, prelude::*, view};

#[component]
pub fn BrokerPoller(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");
    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    let timeout_ms_ref = NodeRef::<Input>::new();

    let submit_action = move |e: SubmitEvent| {
        e.prevent_default();
        poll_broker_action.dispatch(PollBroker {
            broker: node_refs.get_broker(),
            trace_topic: node_refs.get_trace_topic(),
            digitiser_event_topic: node_refs.get_digitiser_event_topic(),
            consumer_group: node_refs.get_consumer_group(),
            poll_broker_timeout_ms: timeout_ms_ref
                .get()
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
