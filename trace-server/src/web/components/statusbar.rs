use cfg_if::cfg_if;
use leptos::{component, prelude::*, view, IntoView};

use crate::{structs::BrokerInfo, web::components::{Panel, Section}};

#[component]
pub fn Status() -> impl IntoView {
    view! {
        <Section name = "Status">
            <DisplayBrokerInfo />
            <Panel name = "Status">
                " "
            </Panel>
            <Panel name = "Progress">
                " "
            </Panel>
        </Section>
    }
}

#[component]
pub fn DisplayBrokerInfo() -> impl IntoView {
    let broker_info = use_context::<BrokerInfo>();
            match broker_info {
                Some(broker_info) => {
                    view!{
                        <Panel name = "Broker Info">
                        <table>
                            <tr class = "header">
                                <td></td>
                                <td>"Count"</td>
                                <td>"From"</td>
                                <td>"To"</td>
                            </tr>
                            <TopicInfo name = "Traces" count = 0 timestamp_from = 0 timestamp_to = 0 />
                            <TopicInfo name = "Eventlists" count = 0 timestamp_from = 0 timestamp_to = 0 />
                        </table>
                        </Panel>
                    }
                },
                None => {
                    view!{
                        <Panel name = "Broker Info">
                        <p>"No Broker Data"</p>
                        </Panel>
                    }
                },
            }
        }

#[component]
pub fn TopicInfo(name: &'static str, count: usize, timestamp_from: usize, timestamp_to: usize) -> impl IntoView {
    view! {
        <tr>
            <td>{name}</td>
            <td>{count}</td>
            <td>{timestamp_from}</td>
            <td>{timestamp_to}</td>
        </tr>
    }
}