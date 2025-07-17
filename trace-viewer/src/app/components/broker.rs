use cfg_if::cfg_if;

use leptos::{component, prelude::*, view, IntoView};
use crate::structs::{BrokerTopicInfo, Topics};
use crate::{DefaultData, structs::BrokerInfo, };

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::finder::{MessageFinder, SearchEngine};
    }
}

//#[cfg(feature = "ssr")]
//use crate::finder::MessageFinder;

use crate::app::components::{Panel, Section};

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
pub fn DisplayBrokerInfo(broker_info: ReadSignal<Option<BrokerInfo>>) -> impl IntoView {
    view! {
        <Panel>
            <div class = "block">
                <div class = "control-box">
                    <input type = "submit" value = "Poll Broker" />
                </div>
                <div class = "control-box">
                    {if let Some(broker_info) = broker_info.get() {
                        view!{
                            <BrokerInfoTable>
                                <TopicInfo name = "Traces" info = broker_info.trace />
                                <TopicInfo name = "Eventlists" info = broker_info.events />
                            </BrokerInfoTable>
                        }.into_any()
                    } else {
                        View::new(()).into_any()
                    }}
                </div>
            </div>
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
async fn poll_broker(broker: String, topics: Topics, poll_broker_timeout_ms: u64) -> Result<Option<BrokerInfo>,ServerFnError> {
    println!("{}", broker);
    let consumer = supermusr_common::create_default_consumer(&broker,&None, &None, &"TraceViewer".to_string(), None).inspect_err(|e|println!("{e:?}"))?;
    println!("{}", broker);
    let searcher = SearchEngine::new(consumer, &topics);
    println!("{}", broker);
    Ok(searcher.poll_broker(poll_broker_timeout_ms).await)
}

#[component]
pub(crate) fn BrokerSetup() -> impl IntoView {
    let default = use_context::<DefaultData>().unwrap_or_default();

    let (broker_info, set_broker_info) = signal::<Option<BrokerInfo>>(None);
    let poll_broker = ServerAction::<PollBroker>::new();

    let broker_node_ref = NodeRef::new();
    let trace_topic_node_ref = NodeRef::new();
    let events_topic_node_ref = NodeRef::new();

    view! {
        <Section name = "Broker">
            <form on:submit = {move |e| {
                e.prevent_default();
                let topics = Option::zip(trace_topic_node_ref.get(), events_topic_node_ref.get())
                .map(
                    |(trace_topic, digitiser_event_topic)|
                    Topics {trace_topic: trace_topic.value(), digitiser_event_topic: digitiser_event_topic.value()}
                );
                if let Some((broker, topics)) = Option::zip(broker_node_ref.get(), topics) { 
                    poll_broker.dispatch(PollBroker {
                        broker: broker.value(),
                        topics,
                        poll_broker_timeout_ms: 1000
                    });
                }
            }}>
                <Panel>
                    <div class = "block">
                        <div class = "control-box">
                            <label for = "broker">
                                "Broker URI: "
                            </label>
                            <input type = "url" id = "broker" value = default.broker node_ref = broker_node_ref />
                        </div>

                        <div class = "control-box">
                            <label for = "trace-topic">
                                "Trace Topic: "
                            </label>
                            <input type = "text" id = "trace-topic" value = default.topics.trace_topic node_ref = trace_topic_node_ref />
                        </div>

                        <div class = "control-box">
                            <label for = "eventlist-topic">
                                "Eventlist Topic: "
                            </label>
                            <input type = "text" id = "eventlist-topic" value = default.topics.digitiser_event_topic node_ref = events_topic_node_ref />
                        </div>
                    </div>
                </Panel>
                <DisplayBrokerInfo broker_info />
            </form>
        </Section>
    }
}
