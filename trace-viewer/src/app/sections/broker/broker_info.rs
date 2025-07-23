
use leptos::{component, prelude::*, view, IntoView};
use crate::app::sections::broker::PollBroker;
use crate::structs::BrokerTopicInfo;
use crate::{structs::BrokerInfo};
use crate::app::components::Panel;

#[component]
fn BrokerInfoHeader() -> impl IntoView {
    view! {
        <tr class = "header">
            <td></td>
            <td>"Count"</td>
            <td>"Date From"</td>
            <td>"Time From"</td>
            <td>"Date To"</td>
            <td>"Time To"</td>
        </tr>
    }
}

#[component]
pub fn TopicInfo(name: &'static str, info: BrokerTopicInfo) -> impl IntoView {
    let from_date = info.timestamps.0.date_naive().format("%Y-%m-%d").to_string();
    let from_time = info.timestamps.0.time().format("%H:%M:%S.%f").to_string();
    
    let to_date = info.timestamps.1.date_naive().format("%Y-%m-%d").to_string();
    let to_time = info.timestamps.1.time().format("%H:%M:%S.%f").to_string();
    view! {
        <tr>
            <td class = "topic-name">{ name }</td>
            <td>{ (info.offsets.1 - info.offsets.0).to_string() }</td>
            <td> {from_date} </td>
            <td> {from_time} </td>
            <td> {to_date} </td>
            <td> {to_time} </td>
        </tr>
    }
}

#[component]
fn BrokerInfoTable(broker_info: BrokerInfo) -> impl IntoView {
    let date = broker_info.timestamp.date_naive().format("%Y-%m-%d").to_string();
    let time = broker_info.timestamp.time().format("%H:%M:%S").to_string();
    view!{
        <h3> "Broker content as of " {date} " " {time} "."</h3>
    
        <table>
            <BrokerInfoHeader />
            <TopicInfo name = "Traces" info = broker_info.trace />
            <TopicInfo name = "Eventlists" info = broker_info.events />
        </table>
    }
}

#[component]
pub fn DisplayBrokerInfo(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    move || if poll_broker_action.pending().get() {
        view!{
            <Panel>
                <p> "Loading Broker Info..."</p>
            </Panel>
        }.into_any()
    } else if let Some(broker_info) = poll_broker_action.value().get() {
        view!{
            <Panel>
                {move || {
                        let broker_info = broker_info.clone();
                        match broker_info {
                            Ok(Some(broker_info)) => {
                                view!{ <BrokerInfoTable broker_info /> }.into_any()
                            },
                            Ok(None) => view!{<h3> "No Broker Data Available" </h3>}.into_any(),
                            Err(e) => view!{<h3> "Server Error:" {e.to_string()} </h3>}.into_any(),
                        }
                    }
                }
            </Panel>
        }.into_any()
    } else {
        view!{
            ""
        }.into_any()
    }
}
