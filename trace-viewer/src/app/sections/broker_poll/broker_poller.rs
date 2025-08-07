use super::PollBroker;
use crate::{
    DefaultData,
    app::components::{InputBoxWithLabel, Panel, SubmitBox},
};
use leptos::{IntoView, component, html::Input, prelude::*, view};

#[component]
pub fn BrokerPoller(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    let timeout_ms_ref = NodeRef::<Input>::new();

    let submit_action = move || {
        poll_broker_action.dispatch(PollBroker {
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
            <form on:submit = move |e| { e.prevent_default(); submit_action() }>
                <InputBoxWithLabel name = "poll_broker_timeout_ms" label = "Poll Broker Timeout (ms):" input_type = "number" value = default.poll_broker_timeout_ms node_ref = timeout_ms_ref />

                <SubmitBox label = "Poll Broker" classes = vec!["across-two-cols"]/>
            </form>
        </Panel>
    }
}
