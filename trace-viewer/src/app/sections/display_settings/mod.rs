use leptos::{IntoView, component, html::Input, prelude::*, view};

use crate::app::components::{InputBoxWithLabel, Section};

#[derive(Default, Clone, Copy)]
pub(crate) struct DisplaySettingsNodeRefs {
    pub(crate) width_ref: NodeRef<Input>,
    pub(crate) height_ref: NodeRef<Input>,
}

#[component]
pub(crate) fn DisplaySettings() -> impl IntoView {
    let display_settings_node_refs = use_context::<DisplaySettingsNodeRefs>()
        .expect("display_settings_node_refs should be provided, this should never fail.");

    view! {
        <Section id = "graph-settings" text = "Graph Settings">
            <InputBoxWithLabel name = "width" label = "Width (px):" input_type = "number" value = "1024" node_ref = display_settings_node_refs.width_ref/>
            <InputBoxWithLabel name = "height" label = "Height (px):" input_type = "number" value = "800" node_ref = display_settings_node_refs.height_ref/>
        </Section>
    }
}
