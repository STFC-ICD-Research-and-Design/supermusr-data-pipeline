use crate::{
    app::{
        main_content::MainLevelContext,
        sections::{
            results::{context::ResultsLevelContext, search_results::SelectTraceLevelContext},
            search::SearchLevelContext,
        },
        server_functions::CreateAndFetchPlotly,
    },
    structs::SelectedTraceIndex,
};
use leptos::{IntoView, component, ev::MouseEvent, prelude::*, view};


#[component]
pub(super) fn SelectChannels(index: usize, mut channels: Vec<u32>) -> impl IntoView {
    let result_level_context = use_context::<ResultsLevelContext>()
        .expect("results_settings_node_refs should be provided, this should never fail.");
    let display_all_channels = result_level_context.display_all_channels.read_only();

    let search_level_context = use_context::<SearchLevelContext>()
        .expect("search_broker_node_refs should be provided, this should never fail.");

    channels.sort();
    view! {
        <div class = "channel-list">
            <For each = move ||channels.clone().into_iter()
                key = ToOwned::to_owned
                let (channel)
            >
                {move || (display_all_channels.get() || search_level_context.channels.get().contains(&channel)).then(
                    || view!{ <Channel this_index_and_channel = SelectedTraceIndex { index, channel } /> }
                )}
            </For>
        </div>
    }
}

#[component]
fn Channel(this_index_and_channel: SelectedTraceIndex) -> impl IntoView {
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