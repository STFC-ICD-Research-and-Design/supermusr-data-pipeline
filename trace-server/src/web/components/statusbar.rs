use leptos::{component, prelude::*, view, IntoView};

use crate::web::components::{Panel, Section};

#[component]
pub(crate) fn TopicInfo(name: &'static str, count: usize, timestamp_from: usize, timestamp_to: usize) -> impl IntoView {
    view! {
        <tr>
            <td>{name}</td>
            <td>{count}</td>
            <td>{timestamp_from}</td>
            <td>{timestamp_to}</td>
        </tr>
    }
}

#[component]
pub(crate) fn BrokerInfo() -> impl IntoView {
    view! {
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
}

#[component]
pub(crate) fn Status() -> impl IntoView {
    view! {
        <Section name = "Status">
            <BrokerInfo />
            <Panel name = "Status">
                " "
            </Panel>
            <Panel name = "Progress">
                " "
            </Panel>
        </Section>
    }
}