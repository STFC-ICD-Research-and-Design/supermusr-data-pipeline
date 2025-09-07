use crate::{
    app::{
        components::toggle_closed, main_content::MainLevelContext, sections::results::results_section::ResultsLevelContext, server_functions::CreateAndFetchPlotly
    },
    structs::{SelectedTraceIndex, TraceSummary},
};
use leptos::{IntoView, component, ev::MouseEvent, prelude::*, view};
use std::collections::BTreeMap;

type TraceSummariesByTime = Vec<(String, Vec<TraceSummary>)>;

fn sort_trace_summaries(trace_summaries: Vec<TraceSummary>) -> Vec<(String, TraceSummariesByTime)> {
    let mut trace_by_date_and_time = BTreeMap::<String, BTreeMap<String, Vec<TraceSummary>>>::new();

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

/// This struct enable a degree of type-checking for the [use_context]/[use_context] functions.
/// Any component making use of the following fields should call `use_context::<SelectTraceLevelContext>()`
/// and select the desired field.
#[derive(Clone)]
struct SelectTraceLevelContext {
    select_trace_index: RwSignal<Option<SelectedTraceIndex>>,
}

#[component]
pub(crate) fn SelectTrace(trace_summaries: Vec<TraceSummary>) -> impl IntoView {
    let trace_by_date_and_time = sort_trace_summaries(trace_summaries);

    provide_context(SelectTraceLevelContext {
        select_trace_index: RwSignal::<Option<SelectedTraceIndex>>::new(None),
    });

    view! {
        <div class = "digitiser-message-list">
            <For
                each = move ||trace_by_date_and_time.clone().into_iter()
                key = |(date,_)|date.clone()
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
                key = |(time,_)|time.clone()
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
fn TraceMessagesByTime(time: String, mut trace_summaries: Vec<TraceSummary>) -> impl IntoView {
    trace_summaries.sort_by(|summary1, summary2| {
        summary1
            .id
            .partial_cmp(&summary2.id)
            .expect("Ordering should complete, this should never fail.")
    });
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
    let selected_trace_index = use_context::<SelectTraceLevelContext>()
        .expect("SelectTraceLevelContext should be provided, this should never fail.")
        .select_trace_index;

    let selected_pred = move || {
        selected_trace_index
            .get()
            .is_some_and(|index_and_channel| index_and_channel.index == trace_summary.index)
    };

    let trace_summary_metadata = trace_summary.clone();

    view! {
        <div class = "digitiser-message" class = ("selected", selected_pred)>
            <div class = "digitiser-message-main">
                <div class = "digitiser-message-id"> "Id: " {trace_summary.id}</div>
                <SelectChannels
                    index = trace_summary.index
                    channels = trace_summary.channels
                />
            </div>
            <Metadata trace_summary = trace_summary_metadata />
        </div>
    }
}

#[component]
pub(crate) fn SelectChannels(index: usize, mut channels: Vec<u32>) -> impl IntoView {
    channels.sort();
    view! {
        <div class = "channel-list">
            <For each = move ||channels.clone().into_iter()
                key = ToOwned::to_owned
                let (channel)
            >
                <Channel this_index_and_channel = SelectedTraceIndex { index, channel } />
            </For>
        </div>
    }
}

#[component]
pub(crate) fn Channel(this_index_and_channel: SelectedTraceIndex) -> impl IntoView {
    let main_context = use_context::<MainLevelContext>()
        .expect("MainLevelContext should be provided, this should never fail.");

    let create_and_fetch_plotly = use_context::<ResultsLevelContext>()
        .expect("ResultsLevelContext should be provided, this should never fail.")
        .create_and_fetch_plotly;

    let selected_trace_index = use_context::<SelectTraceLevelContext>()
        .expect("SelectTraceLevelContext should be provided, this should never fail.")
        .select_trace_index;

    let uuid = main_context.uuid;

    let SelectedTraceIndex { index, channel } = this_index_and_channel.clone();

    let on_click = {
        let this_index_and_channel = this_index_and_channel.clone();
        move |_: MouseEvent| {
            if let Some(uuid) = uuid.get() {
                selected_trace_index.set(Some(this_index_and_channel.clone()));
                create_and_fetch_plotly.dispatch(CreateAndFetchPlotly {
                    uuid,
                    index_and_channel: this_index_and_channel.clone(),
                });
            }
        }
    };

    let selected_pred = move || {
        selected_trace_index.get().is_some_and(|index_and_channel| {
            index_and_channel.index == index && index_and_channel.channel == channel
        })
    };

    view! {
        <div class = "channel"
            class = ("selected", selected_pred)
            on:click = on_click
        >
            {this_index_and_channel.channel}
        </div>
    }
}

#[component]
fn Metadata(trace_summary: TraceSummary) -> impl IntoView {
    view!{
        <div class = "digitiser-message-metadata closable-container closed">
            <div class = "digitiser-message-metadata-title closable-control"
                    on:click:target = move |e| toggle_closed(e.target().parent_element())>
                "Digitiser Metadata"
            </div>
            <div class = "digitiser-message-metadata-content closable">
              <div> "Frame Number: "      {trace_summary.frame_number} </div>
              <div> "Period Number: "     {trace_summary.period_number} </div>
              <div> "Protons per Pulse: " {trace_summary.protons_per_pulse} </div>
              <div> "Running: "           {trace_summary.running} </div>
              <div> "VetoFlags: "         {trace_summary.veto_flags} </div>
            </div>
        </div>
    }
}