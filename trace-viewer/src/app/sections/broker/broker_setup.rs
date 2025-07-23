use crate::{app::components::{ControlBox, ControlBoxWithLabel, Panel, VerticalBlock}, DefaultData};
use leptos::{component, prelude::*, view, IntoView};
use super::PollBroker;

#[component]
pub fn BrokerSetup(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");
    view! {
        <ActionForm action = poll_broker_action>
            <Panel>
                <VerticalBlock>
                    <ControlBoxWithLabel name = "broker" label = "Broker URI: ">
                        <input type = "text" id = "broker" value = default.broker />
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "trace_topic" label = "Trace Topic: ">
                        <input type = "text" id = "trace_topic" value = default.topics.trace_topic />
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "digitiser_event_topic" label = "Eventlist Topic: ">
                        <input type = "text" id = "digitiser_event_topic" value = default.topics.digitiser_event_topic />
                    </ControlBoxWithLabel>
                </VerticalBlock>
            </Panel>
            <Panel>
                <VerticalBlock>
                    <ControlBoxWithLabel name = "consumer-group" label = "Consumer Group: ">
                        <input type = "text" id = "consumer-group" value = default.consumer_group />
                    </ControlBoxWithLabel>

                    <ControlBoxWithLabel name = "poll-broker-timeout-ms" label = "Poll Broker Timeout (ms): ">
                        <input type = "number" id = "poll-broker-timeout-ms" value = default.poll_broker_timeout_ms />
                    </ControlBoxWithLabel>

                    <ControlBox>
                        <input type = "submit" value = "Poll Broker" />
                    </ControlBox>
                </VerticalBlock>
            </Panel>
        </ActionForm>
    }
}