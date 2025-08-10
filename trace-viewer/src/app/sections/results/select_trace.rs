use std::collections::HashMap;

use leptos::{IntoView, component, ev::MouseEvent, prelude::*, view};

use crate::{
    app::{main_content::MainLevelContext, server_functions::CreateAndFetchPlotly, Uuid},
    structs::{SelectedTraceIndex, TraceSummary},
};

fn sort_trace_summaries(
    trace_summaries: Vec<TraceSummary>,
) -> Vec<(String, Vec<(String, Vec<TraceSummary>)>)> {
    let mut trace_by_date_and_time = HashMap::<String, HashMap<String, Vec<TraceSummary>>>::new();

    for trace_summary in trace_summaries.into_iter() {
        trace_by_date_and_time
            .entry(trace_summary.date.clone())
            .or_default()
            .entry(trace_summary.time.clone())
            .or_default()
            .push(trace_summary);
    }

    trace_by_date_and_time
        .into_iter()
        .map(|(date, by_time)| (date, by_time.into_iter().collect::<Vec<_>>()))
        .collect::<Vec<_>>()
}

#[component]
pub(crate) fn SelectTrace(trace_summaries: Vec<TraceSummary>) -> impl IntoView {
    let trace_by_date_and_time = sort_trace_summaries(trace_summaries);

    provide_context(RwSignal::<Option<SelectedTraceIndex>>::new(None));

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
    view! {
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
fn TraceMessagesByTime(time: String, trace_summaries: Vec<TraceSummary>) -> impl IntoView {
    view! {
        <div class = "digitiser-messages-by-time">
            <div class = "digitiser-messages-time"> "Time: " {time} </div>
            <For
                each = move ||trace_summaries.clone().into_iter()
                key = TraceSummary::to_owned
                let(trace_summary)
            >
                <TraceMessage trace_summary />
            </For>
        </div>
    }
}

#[component]
fn TraceMessage(trace_summary: TraceSummary) -> impl IntoView {
    let selected_trace_index = use_context::<RwSignal::<Option<SelectedTraceIndex>>>()
        .expect("SelectedTraceIndex should be provided, this should never fail.");

    let selected_pred = move ||
        selected_trace_index
            .get()
            .is_some_and(|index_and_channel| index_and_channel.index == trace_summary.index);

    view! {
        <div class = "digitiser-message" class = ("selected", selected_pred)>
            <div class = "digitiser-message-id"> "Id: " {trace_summary.id}</div>
            <SelectChannels
                index = trace_summary.index
                channels = trace_summary.channels
            />
        </div>
    }
}

#[component]
pub(crate) fn SelectChannels(index: usize, mut channels: Vec<u32>) -> impl IntoView {
    channels.sort();
    view! {
        <div class = "channel-list">
            <For each = move ||channels.clone().into_iter()
                key = u32::to_owned
                let (channel)
            >
                <Channel index_and_channel = SelectedTraceIndex { index, channel } />
            </For>
        </div>
    }
}

#[component]
pub(crate) fn Channel(index_and_channel: SelectedTraceIndex) -> impl IntoView {
    let main_context = use_context::<MainLevelContext>().expect("");

    let create_and_fetch_plotly =
        use_context::<ServerAction<CreateAndFetchPlotly>>().expect("");
        
    let selected_trace_index = use_context::<RwSignal::<Option<SelectedTraceIndex>>>()
        .expect("SelectedTraceIndex should be provided, this should never fail.");

    let uuid = main_context.uuid;

    let SelectedTraceIndex { index, channel } = index_and_channel.clone();

    let on_click = {
        let index_and_channel = index_and_channel.clone();
        move |_: MouseEvent| {
            if let Some(uuid) = uuid.get() {
                selected_trace_index.set(Some(index_and_channel.clone()));
                create_and_fetch_plotly.dispatch(
                    CreateAndFetchPlotly {
                        uuid,
                        index_and_channel: index_and_channel.clone(),
                    },
                );
            }
        }
    };

    let selected_pred = move ||
        selected_trace_index
            .get()
            .is_some_and(|index_and_channel|
                index_and_channel.index == index && index_and_channel.channel == channel);

    view! {
        <div class = "channel"
            class = ("selected", selected_pred)
            on:click = on_click
        >
            {index_and_channel.channel}
        </div>
    }
}
