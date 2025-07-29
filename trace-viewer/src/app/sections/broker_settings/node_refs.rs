use leptos::{html::Input, prelude::*};

#[derive(Default, Clone)]
pub struct BrokerSettingsNodeRefs {
    pub broker: NodeRef<Input>,
    pub trace_topic: NodeRef<Input>,
    pub digitiser_event_topic: NodeRef<Input>,
    pub consumer_group: NodeRef<Input>,
}

impl BrokerSettingsNodeRefs {
    pub(crate) fn get_broker(&self) -> String {
        self.broker
            .get()
            .expect("broker node_ref should exist, this should never fail.")
            .value()
    }

    pub(crate) fn get_trace_topic(&self) -> String {
        self.trace_topic
            .get()
            .expect("trace_topic node_ref should exist, this should never fail.")
            .value()
    }

    pub(crate) fn get_digitiser_event_topic(&self) -> String {
        self.digitiser_event_topic
            .get()
            .expect("digitiser_event_topic node_ref should exist, this should never fail.")
            .value()
    }

    pub(crate) fn get_consumer_group(&self) -> String {
        self.consumer_group
            .get()
            .expect("consumer_group node_ref should exist, this should never fail.")
            .value()
    }
}
