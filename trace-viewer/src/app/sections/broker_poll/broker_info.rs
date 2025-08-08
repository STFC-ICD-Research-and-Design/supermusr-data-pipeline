use crate::app::components::DisplayErrors;
use crate::app::server_functions::PollBroker;
use crate::structs::BrokerInfo;
use crate::structs::BrokerTopicInfo;
use leptos::either::Either;
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub fn DisplayBrokerInfo(poll_broker_action: ServerAction<PollBroker>) -> impl IntoView {
    move || {
        if poll_broker_action.pending().get() {
            Either::Left(view! {<p> "Loading Broker Info..."</p>})
        } else {
            Either::Right(poll_broker_action.value().get().map(move |broker_info| {
                let broker_info = broker_info.map(|broker_info| view! { <BrokerInfoTable broker_info /> });
                view! {
                    <ErrorBoundary fallback = move |errors| view!{ <DisplayErrors errors /> }>
                        {broker_info}
                    </ErrorBoundary>
                }
            }))
        }
    }
}


#[component]
fn BrokerInfoTable(broker_info: BrokerInfo) -> impl IntoView {
    let date = broker_info
        .timestamp
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let time = broker_info.timestamp.time().format("%H:%M:%S").to_string();
    view! {
        <div class = "broker-info">
            <div class = "table">
                <TopicInfo name = "Traces" info = broker_info.trace />
                <TopicInfo name = "Event Lists" info = broker_info.events />
            </div>

            <div class = "broker-info-status">
                "Last refreshed: " {date} " " {time} "."
            </div>
        </div>
    }
}

#[component]
pub fn TopicInfo(name: &'static str, info: BrokerTopicInfo) -> impl IntoView {
    match info.timestamps {
        Some((from, to)) => {
            let from_date = from.date_naive().format("%Y-%m-%d").to_string();
            let from_time = from.time().format("%H:%M:%S.%f").to_string();

            let to_date = to.date_naive().format("%Y-%m-%d").to_string();
            let to_time = to.time().format("%H:%M:%S.%f").to_string();
            view! {
                <div class = "topic-name">{name}</div>
                <div class = "topic-data-header">"Count"</div>
                <div class = "topic-data-header">"Date From"</div>
                <div class = "topic-data-header">"Time From"</div>
                <div class = "topic-data-header">"Date To"</div>
                <div class = "topic-data-header">"Time To"</div>
                <div class = "topic-data-item">{ (info.offsets.1 - info.offsets.0).to_string() }</div>
                <div class = "topic-data-item"> {from_date} </div>
                <div class = "topic-data-item"> {from_time} </div>
                <div class = "topic-data-item"> {to_date} </div>
                <div class = "topic-data-item"> {to_time} </div>
            }.into_any()
        }
        None => view! {
            <div class = "topic-name">{name}</div>
            <div class = "topic-data-unavailable"> "No messages on topic" </div>
        }
        .into_any(),
    }
}
