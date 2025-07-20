use leptos::ev::SubmitEvent;
use leptos::html::Input;
use leptos::{component, prelude::*, view, IntoView};
use crate::app::PollBrokerData;
use crate::structs::{BrokerTopicInfo, Topics};
use crate::{DefaultData, structs::BrokerInfo};
use crate::app::components::{ControlBox, ControlBoxWithLabel, Panel, Section, VerticalBlock};
use tracing::{debug, instrument};

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
fn BrokerInfoTable(broker_info: Result<Option<BrokerInfo>,ServerFnError>) -> impl IntoView {
    match broker_info {
        Ok(Some(broker_info)) => {
            let date = broker_info.timestamp.date_naive().format("%Y-%m-%d").to_string();
            let time = broker_info.timestamp.time().format("%H:%M:%S").to_string();
            view!{
                <h3>"Broker content as of " {date} " " {time} "."</h3>
                <table>
                    <BrokerInfoHeader />
                    <TopicInfo name = "Traces" info = broker_info.trace />
                    <TopicInfo name = "Eventlists" info = broker_info.events />
                </table>
            }.into_any()
        },
        Ok(None) => view!{<h3> "No Broker Data Available" </h3>}.into_any(),
        Err(e) => view!{<h3> "Server Error:" {e.to_string()} </h3>}.into_any(),
    }
}

#[component]
pub fn DisplayBrokerInfo(broker_info: Result<Option<BrokerInfo>,ServerFnError>) -> impl IntoView {
    view! {
        <Panel>
            <VerticalBlock>
                <ControlBox>
                    <BrokerInfoTable broker_info />
                </ControlBox>
            </VerticalBlock>
        </Panel>
    }
}

#[server]
#[instrument(skip_all)]
async fn poll_broker(poll_broker_data: PollBrokerData) -> Result<Option<BrokerInfo>,ServerFnError> {
    use crate::finder::{MessageFinder, SearchEngine};

    debug!("{poll_broker_data:?}");
    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    debug!("{default:?}");

    let consumer = supermusr_common::create_default_consumer(
        poll_broker_data.broker.as_ref().unwrap_or_else(||&default.broker),
        &default.username,
        &default.password,
        &default.consumer_group,
        None)?;

    let searcher = SearchEngine::new(
        consumer,
        &poll_broker_data.topics.unwrap_or_else(||default.topics)
    );

    let broker_info = searcher
        .poll_broker(default.poll_broker_timeout_ms)
        .await;
    Ok(broker_info)
}

#[component]
pub(crate) fn BrokerSetup() -> impl IntoView {    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    let broker_node_ref = NodeRef::<Input>::new();
    let trace_topic_node_ref = NodeRef::<Input>::new();
    let events_topic_node_ref = NodeRef::<Input>::new();
    let consumer_group_node_ref = NodeRef::<Input>::new();
    let poll_broker_timeout_node_ref = NodeRef::<Input>::new();

    let package_topics_fn = move ||
        Option::zip(trace_topic_node_ref.get(), events_topic_node_ref.get())
            .map(|(trace_topic, digitiser_event_topic)|
                Topics {trace_topic: trace_topic.value(), digitiser_event_topic: digitiser_event_topic.value()}
            );

    let broker_info_poll_fn = move |_: &()| {
        poll_broker(PollBrokerData {
            broker: broker_node_ref.get().map(|broker|broker.value()),
            topics: package_topics_fn(),
        })
    };
    let broker_info_action = Action::new(broker_info_poll_fn);
    let broker_info = broker_info_action.value().read_only();

    let on_submit = move |e : SubmitEvent| {
        e.prevent_default();
        broker_info_action.dispatch(());
    };

    view! {
        <form on:submit = on_submit>
            <Section name = "Broker">
                <Panel>
                    <VerticalBlock>
                        <ControlBoxWithLabel name = "broker" label = "Broker URI: ">
                            <input type = "text" id = "broker" value = default.broker node_ref = broker_node_ref />
                        </ControlBoxWithLabel>

                        <ControlBoxWithLabel name = "trace-topic" label = "Trace Topic: ">
                            <input type = "text" id = "trace-topic" value = default.topics.trace_topic node_ref = trace_topic_node_ref />
                        </ControlBoxWithLabel>

                        <ControlBoxWithLabel name = "eventlist-topic" label = "Eventlist Topic: ">
                            <input type = "text" id = "eventlist-topic" value = default.topics.digitiser_event_topic node_ref = events_topic_node_ref />
                        </ControlBoxWithLabel>
                    </VerticalBlock>
                </Panel>
                <Panel>
                    <VerticalBlock>
                        <ControlBoxWithLabel name = "consumer-group" label = "Consumer Group: ">
                            <input type = "text" id = "consumer-group" value = default.consumer_group node_ref = consumer_group_node_ref />
                        </ControlBoxWithLabel>

                        <ControlBoxWithLabel name = "poll-broker-timeout-ms" label = "Poll Broker Timeout (ms): ">
                            <input type = "number" id = "poll-broker-timeout-ms" value = default.poll_broker_timeout_ms node_ref = poll_broker_timeout_node_ref />
                        </ControlBoxWithLabel>

                        <ControlBox>
                            <input type = "submit" value = "Poll Broker" />
                        </ControlBox>
                    </VerticalBlock>
                </Panel>
                {move ||
                    broker_info.get().map(|broker_info|view!{
                        <DisplayBrokerInfo broker_info />
                    })
                }
                
            </Section>
        </form>
    }
}
