use leptos::{component, html::Input, prelude::*, view, IntoView};

use crate::app::{components::{ControlBoxWithLabel, InputBoxWithLabel, Panel, Section}, sections::{search::SearchBroker, Display}};


#[component]
pub(crate) fn SearchResults(search_broker_action: ServerAction<SearchBroker>) -> impl IntoView {
    move || if search_broker_action.pending().get() {
        view!{
            <Panel>
                <p> "Searching Broker..."</p>
            </Panel>
        }.into_any()
    } else if let Some(search_results) = search_broker_action.value().get() {
        match search_results {
            Ok(search_results) => {

                let digitiser_messages = search_results.cache.iter().map(ToOwned::to_owned).collect::<Vec<_>>();
                let result_summaries = digitiser_messages.iter().map(|(metadata,_)| (metadata.timestamp.format("%y-%m-%d %H:%M:%S.%f").to_string(), metadata.id)).collect::<Vec<_>>();

                let (selected_message_index, set_selected_message_index) = signal::<Option<usize>>(None);
                let (selected_channel, set_selected_channel) = signal::<Option<u32>>(None);
                let selected_trace = {
                    let digitiser_messages = digitiser_messages.clone();
                    move || selected_message_index.get()
                    .and_then(|index|digitiser_messages.get(index))
                    .and_then(|(_,trace)|
                        selected_channel.get()
                            .map(move |channel|trace.traces[&channel].clone())
                        )
                };

                let width_ref = NodeRef::<Input>::new();
                let height_ref = NodeRef::<Input>::new();

                view!{
                    <Section name = "Results">
                        <Panel>
                            <SelectTrace result_summaries = result_summaries selected_message_index set_selected_message_index/>
                            {
                                let digitiser_messages = digitiser_messages.clone();
                                move ||selected_message_index.get()
                                    .and_then(|index|digitiser_messages.get(index))
                                    .map(move |(_, dig_msg)| {
                                            let channels = dig_msg.traces.keys().copied().collect::<Vec<_>>();
                                            view!{
                                                <SelectChannels channels selected_channel set_selected_channel width_ref height_ref />
                                            }
                                    })
                            }
                        </Panel>
                        <Display selected_trace width_ref height_ref/>
                    </Section>
                }.into_any()
            },
            Err(e) => view!{<h3> "Server Error:" {e.to_string()} </h3>}.into_any(),
        }
    } else {
        view!{""}.into_any()
    }
}


#[component]
pub(crate) fn SelectTrace(result_summaries: Vec<(String, u8)>, selected_message_index: ReadSignal<Option<usize>>, set_selected_message_index: WriteSignal<Option<usize>>) -> impl IntoView {
    view! {
        <Panel>
            <For 
                each = move ||result_summaries.clone().into_iter().enumerate()
                key = |summary|summary.clone()
                let((idx, (timestamp, digitiser_id)))
            >
                <div class = "message-list">
                    <div class = "message-option"
                        class:message_selected = move||selected_message_index.get().is_some_and(|index|index==idx)
                        on:click = move |_|set_selected_message_index.set(Some(idx))
                    >
                        <div class = "message-option-header">"Timestamp:"</div>
                        <div class = "message-option-header">"Digitiser ID:"</div>
                        <div class = "message-option-data">{timestamp}</div>
                        <div class = "message-option-data">{digitiser_id}</div>
                    </div>
                </div>
            </For>
        </Panel>
    }
}

#[component]
pub(crate) fn SelectChannels(channels: Vec<u32>, selected_channel: ReadSignal<Option<u32>>, set_selected_channel: WriteSignal<Option<u32>>, width_ref: NodeRef<Input>, height_ref: NodeRef<Input>) -> impl IntoView {
    view!{
        <Panel>
            <ControlBoxWithLabel name = "channels" label = "Channels:">
                <For
                    each = {
                        let channels = channels.clone();
                        move ||channels.clone().into_iter()
                    }
                    key = |key|key.clone()
                    let (channel)
                >
                    <div class = "channel-option"
                        class:channel_selected = move||selected_channel.get().is_some_and(|index|index==channel)
                        on:click = move |_| set_selected_channel.set(Some(channel))
                    >
                        {channel}
                    </div>
                </For>
            </ControlBoxWithLabel>

            <InputBoxWithLabel name = "width" label = "Width (px):" input_type = "number" value = "1024" node_ref = width_ref/>

            <InputBoxWithLabel name = "height" label = "Height (px):" input_type = "number" value = "800" node_ref = height_ref/>
        </Panel>
    }
}