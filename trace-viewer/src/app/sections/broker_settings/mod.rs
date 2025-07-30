mod node_refs;

use crate::{
    DefaultData,
    app::components::{InputBoxWithLabel, Panel, Section},
};
use leptos::{IntoView, component, prelude::*, view};

pub(crate) use node_refs::BrokerSettingsNodeRefs;

#[component]
pub fn BrokerSetup() -> impl IntoView {
    let node_refs = use_context::<BrokerSettingsNodeRefs>()
        .expect("Node refs should be available, this should never fail.");

    let default = use_context::<DefaultData>()
        .expect("Default Data should be availble, this should never fail.");
    view! {
        <Section name = "Broker Settings">
            <Panel classes = vec!["broker-setup"]>
                <InputBoxWithLabel name = "broker" label = "Broker URI: " input_type = "text" value = default.broker node_ref = node_refs.broker />
                <InputBoxWithLabel name = "trace_topic" label = "Trace Topic:" input_type = "text" value = default.topics.trace_topic node_ref = node_refs.trace_topic />
                <InputBoxWithLabel name = "digitiser_event_topic" label = "Eventlist Topic:" input_type = "text" value = default.topics.digitiser_event_topic node_ref = node_refs.digitiser_event_topic />
                <InputBoxWithLabel name = "consumer_group" label = "Consumer Group:" input_type = "text" value = default.consumer_group node_ref = node_refs.consumer_group />
            </Panel>
        </Section>
    }
}
