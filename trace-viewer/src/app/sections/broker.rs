use leptos::ev::SubmitEvent;
use leptos::html::Input;
use leptos::task::spawn_local;
use leptos::{component, prelude::*, view, IntoView};
use crate::app::PollBrokerData;
use crate::structs::{BrokerTopicInfo, Topics};
use crate::{DefaultData, structs::BrokerInfo};
use crate::app::components::{ControlBox, ControlBoxWithLabel, Panel, Section, VerticalBlock};

#[component]
fn BrokerInfoTable(children: Children) -> impl IntoView {
    view!{
        <table>
            <tr class = "header">
                <td></td>
                <td>"Count"</td>
                <td>"From"</td>
                <td>"To"</td>
            </tr>
            {children()}
        </table>
    }
}

#[component]
pub fn DisplayBrokerInfo(broker_info: ReadSignal<Option<Result<Option<BrokerInfo>,ServerFnError>>>) -> impl IntoView {
    view! {
        <Panel>
            <VerticalBlock>
                <ControlBox>
                    <input type = "submit" value = "Poll Broker" />
                </ControlBox>
                <ControlBox>
                    {move ||if let Some(broker_info) = broker_info.get() {
                        match broker_info {
                            Ok(Some(broker_info)) => view!{
                                    <BrokerInfoTable>
                                        <TopicInfo name = "Traces" info = broker_info.trace />
                                        <TopicInfo name = "Eventlists" info = broker_info.events />
                                    </BrokerInfoTable>
                                }.into_any(),
                            Ok(None) => view!{"Inner Missing"}.into_any(),
                            Err(e) => view!{"Error"}.into_any(),
                        }
                    } else {
                        view!{"Outer Missing"}.into_any()
                    }}
                </ControlBox>
            </VerticalBlock>
        </Panel>
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

#[server]
async fn poll_broker(data: PollBrokerData) -> Result<Option<BrokerInfo>,ServerFnError> {
    use crate::finder::{MessageFinder, SearchEngine};
    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    println!("default: {:?}", default);
    println!("poll_broker: {:?}", data);
    let consumer = supermusr_common::create_default_consumer(data.broker.as_ref().unwrap_or_else(||&default.broker),&default.username, &default.password, &default.consumer_group, None).inspect_err(|e|println!("{e:?}"))?;
    let searcher = SearchEngine::new(consumer, &data.topics.unwrap_or_else(||default.topics));
    let broker_info = searcher.poll_broker(default.poll_broker_timeout_ms).await;
    Ok(broker_info.inspect(|info|println!("broker_info: {info:?}")))
}

#[component]
pub(crate) fn BrokerSetup() -> impl IntoView {    
    let default = use_context::<DefaultData>().expect("Default Data should be availble, this should never fail.");

    let (broker_info, set_broker_info) = signal::<Option<BrokerInfo>>(None);

    let broker_node_ref = NodeRef::<Input>::new();
    let trace_topic_node_ref = NodeRef::<Input>::new();
    let events_topic_node_ref = NodeRef::<Input>::new();

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
    let broker_info_signal = broker_info_action.value().read_only();

    view! {
        <form on:submit = {move |e : SubmitEvent| {
            e.prevent_default();
            broker_info_action.dispatch(());
        }}>
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
                <DisplayBrokerInfo broker_info=broker_info_signal />
            </Section>
        </form>
    }
}
