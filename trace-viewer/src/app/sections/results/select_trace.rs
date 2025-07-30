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
    selected_message: ReadSignal<Option<(usize, u32)>>,
    set_selected_message: WriteSignal<Option<(usize, u32)>>,
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
        <div class = "message-list">
            <For
                each = move ||trace_by_date_and_time.clone().into_iter()
                key = |by_date|by_date.clone()
                let((date, trace_summaries_by_time))
            >
                <div class = "message-option-date">"Date: " {date}</div>
                <For
                    each = move ||trace_summaries_by_time.clone().into_iter()
                    key = |by_time|by_time.clone()
                    let((time, trace_summaries))
                >
                    <div class = "message-option-time">"Time: " {time}</div>
                        <TraceMessageOptions
                            trace_summaries
                            selected_message set_selected_message
                        />
                </For>
            </For>
        </div>
    }
}

#[component]
pub(crate) fn TraceMessageOptions(
    trace_summaries: Vec<TraceSummary>,
    selected_message: ReadSignal<Option<(usize, u32)>>,
    set_selected_message: WriteSignal<Option<(usize, u32)>>,
) -> impl IntoView {
    view! {
        <div class = "message-option">
            <div class = "message-option-heading"> "Digitiser Id: " </div>
            <div class = "message-option-heading"> "Channels: " </div>
            <For
                each = move ||trace_summaries.clone().into_iter()
                key = |trace_summary|trace_summary.clone()
                let(trace_summary)
            >
                <div class = "message-option-id"> {trace_summary.id} </div>
                <SelectChannels
                    index = trace_summary.index
                    channels = trace_summary.channels
                    selected_message set_selected_message
                />
            </For>
        </div>
    }
}

#[component]
pub(crate) fn SelectChannels(
    index: usize,
    mut channels: Vec<u32>,
    selected_message: ReadSignal<Option<(usize, u32)>>,
    set_selected_message: WriteSignal<Option<(usize, u32)>>,
) -> impl IntoView {
    channels.sort();
    view! {
        <div class = "channel-list">
            <For
                each = {
                    let channels = channels.clone();
                    move ||channels.clone().into_iter()
                }
                key = |key|key.clone()
                let (channel)
            >
                <div class = "channel-option"
                    class = ("channel-selected", move || selected_message.get().is_some_and(|(idx,ch)|idx==index && ch==channel))
                    on:click = move |_| set_selected_message.set(Some((index,channel)))
                >
                    {channel}
                </div>
            </For>
        </div>
    }
}
