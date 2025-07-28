use leptos::{component, html::Input, prelude::*, view, IntoView};

use crate::app::{components::{ControlBoxWithLabel, InputBoxWithLabel, Panel, Section}, sections::{search::SearchBroker, Display}};

#[derive(Default, Copy)]
pub(crate) struct DisplaySettingsNodeRefs {
    width_ref: NodeRef<Input>,
    height_ref: NodeRef<Input>
}

#[component]
pub(crate) fn DisplaySettings(display_settings_node_refs: DisplaySettingsNodeRefs) -> impl IntoView {
    view!{
        <Section name = "Graph Settings">
            <Panel>
                <InputBoxWithLabel name = "width" label = "Width (px):" input_type = "number" value = "1024" node_ref = display_settings_node_refs.width_ref/>

                <InputBoxWithLabel name = "height" label = "Height (px):" input_type = "number" value = "800" node_ref = display_settings_node_refs.height_ref/>
            </Panel>
        </Section>
    }
}