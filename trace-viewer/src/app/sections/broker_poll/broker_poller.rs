use crate::{
    DefaultData,
    app::{
        components::{InputBoxWithLabel, SubmitBox},
        server_functions::PollBroker,
    },
};
use leptos::{IntoView, component, html::Input, prelude::*, view};

#[component]
pub fn BrokerPoller(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");

    let timeout_ms_ref = NodeRef::<Input>::new();

    view! {
        <ActionForm action = poll_broker_action>
            <div class = "broker-poll">
                <label class = "panel-item" for = "poll_broker_timeout_ms">
                    "Poll Broker Timeout (ms):"
                    <input class = "small" name = "poll_broker_timeout_ms" id = "poll_broker_timeout_ms" value = default.poll_broker_timeout_ms type = "number" node_ref = timeout_ms_ref />
                </label>
                <input type = "submit" value = "Poll Broker"/>
            </div>
        </ActionForm>
    }
}
