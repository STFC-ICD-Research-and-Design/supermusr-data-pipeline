use std::collections::HashMap;

use leptos::{IntoView, component, prelude::*, view};

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct TraceSummary {
    pub(crate) date: String,
    pub(crate) time: String,
    pub(crate) id: u8,
    pub(crate) channels: Vec<u32>,
    pub(crate) index: usize,
}

#[component]
pub(crate) fn SelectTrace(
    trace_summaries: Vec<TraceSummary>,
) -> impl IntoView {
    let mut trace_by_date_and_time = HashMap::<String, HashMap<String, Vec<TraceSummary>>>::new();
    for trace_summary in trace_summaries.into_iter() {
        trace_by_date_and_time
            .entry(trace_summary.date.clone())
            .or_default()
            .entry(trace_summary.time.clone())
            .or_default()
            .push(trace_summary);
    }
    let trace_by_date_and_time = trace_by_date_and_time
        .into_iter()
        .map(|(date, by_time)| (date, by_time.into_iter().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    view! {
        <div class = "digitiser-message-list">
            <For
                each = move ||trace_by_date_and_time.clone().into_iter()
                key = |by_date|by_date.clone()
                let((date, trace_summaries_by_time))>
                    <TraceMessagesByDate date trace_summaries_by_time/>
            </For>
        </div>
    }
}

#[component]
fn TraceMessagesByDate(
    date: String,
    trace_summaries_by_time: Vec<(String, Vec<TraceSummary>)>,
) -> impl IntoView {
    view!{
        <div class = "digitiser-messages-by-date">
            <div class = "digitiser-messages-date"> "Date: " {date} </div>
            <For
                each = move ||trace_summaries_by_time.clone().into_iter()
                key = |by_time|by_time.clone()
                let((time, trace_summaries))
            >
                    <TraceMessagesByTime
                        time
                        trace_summaries
                    />
            </For>
        </div>
    }
}

#[component]
fn TraceMessagesByTime(
    time: String,
    trace_summaries: Vec<TraceSummary>,
) -> impl IntoView {
    view! {
        <div class = "digitiser-messages-by-time">
            <div class = "digitiser-messages-time"> "Time: " {time} </div>
            <For
                each = move ||trace_summaries.clone().into_iter()
                key = |trace_summary|trace_summary.clone()
                let(trace_summary)
            >
                <TraceMessage trace_summary />
            </For>
        </div>
    }
}


#[component]
fn TraceMessage(
    trace_summary: TraceSummary
) -> impl IntoView {
    let (selected_message, _) = use_context::<(ReadSignal<Option<(usize, u32)>>, WriteSignal<Option<(usize, u32)>>)>()
            .expect("");
    view! {
        <div class = "digitiser-message"
            class = ("selected", move || selected_message.get().is_some_and(|(idx,_)|idx==trace_summary.index))
            >
            <div class = "digitiser-message-id"> "Id: " {trace_summary.id}</div>
            <SelectChannels
                index = trace_summary.index
                channels = trace_summary.channels
            />
        </div>
    }
}

#[component]
pub(crate) fn SelectChannels(
    index: usize,
    mut channels: Vec<u32>,
) -> impl IntoView {
    channels.sort();
    view! {
        <div class = "channel-list">
            <For
                each =  move ||channels.clone().into_iter()
                key = |&key|key
                let (channel)
            >
                <Channel index channel />
            </For>
        </div>
    }
}

#[component]
pub(crate) fn Channel(
    index: usize,
    channel: u32,
) -> impl IntoView {
    let (selected_message, set_selected_message) = use_context::<(ReadSignal<Option<(usize, u32)>>, WriteSignal<Option<(usize, u32)>>)>()
            .expect("");
        
    view! {
        <div class = "channel"
            class = ("selected", move || selected_message.get().is_some_and(|(idx,ch)|idx==index && ch==channel))
            on:click = move |_| set_selected_message.set(Some((index,channel)))
        >
            {channel}
        </div>
    }
}