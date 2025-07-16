use chrono::Utc;
use leptos::{component, prelude::*, view, IntoView};

use crate::structs::BrokerTopicInfo;
use crate::{DefaultData, structs::BrokerInfo, };

//#[cfg(feature = "ssr")]
//use crate::finder::MessageFinder;

use crate::app::components::{Panel, Section};

#[component]
pub fn DisplayBrokerInfo() -> impl IntoView {
    let broker_info = use_context::<BrokerInfo>();
            match broker_info {
                Some(broker_info) => {
                    view!{
                        <Panel name = "Broker Info">
                        <div class = "block">
                        <button type = "submit">"Poll Broker"</button>
                        <div>
                        <table>
                            <tr class = "header">
                                <td></td>
                                <td>"Count"</td>
                                <td>"From"</td>
                                <td>"To"</td>
                            </tr>
                            <TopicInfo name = "Traces" info = broker_info.trace />
                            <TopicInfo name = "Eventlists" info = broker_info.events />
                        </table>
                        </div>
                        </div>
                        </Panel>
                    }
                },
                None => {
                    view!{
                        <Panel name = "Broker Info">
                        <div class = "block">
                        <button type = "submit">"Poll Broker"</button>
                        </div>
                        <p>"No Broker Data"</p>
                        </Panel>
                    }
                },
            }
        }

#[component]
pub fn TopicInfo(name: &'static str, info: BrokerTopicInfo) -> impl IntoView {
    view! {
        <tr>
            <td>{ name }</td>
            <td>{ (info.offsets.1 - info.offsets.0).to_string() }</td>
            <td>{ info.timestamps.0.to_string() }</td>
            <td>{ info.timestamps.1.to_string() }</td>
        </tr>
    }
}

#[component]
pub(crate) fn BrokerSetup() -> impl IntoView {
    let default = use_context::<DefaultData>().unwrap_or_default();

    view! {
        <Section name = "Broker">
            <Panel name = "Broker">
                <div class = "block">
                    <div class = "setup-control">
                        <label for = "broker">
                            "Broker URI: "
                            <input type = "url" id = "broker" value = default.broker/>
                        </label>
                    </div>

                    <div class = "setup-control">
                        <label for = "trace-topic">
                            "Trace Topic: "
                            <input type = "text" id = "trace-topic" value = default.topics.trace_topic />
                        </label>
                    </div>

                    <div class = "setup-control">
                        <label for = "eventlist-topic">
                            "Eventlist Topic: "
                            <input type = "text" id = "eventlist-topic" value = default.topics.digitiser_event_topic />
                        </label>
                    </div>
                </div>
            </Panel>
            <DisplayBrokerInfo />
        </Section>
    }
}
