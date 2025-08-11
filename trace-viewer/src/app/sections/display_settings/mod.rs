use crate::app::components::Section;
use leptos::{IntoView, component, html::Input, prelude::*, view};

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
            <label for = "width">
                "Width (px):"
                <input name = "width" id = "width" value = 1024 type = "number" node_ref = display_settings_node_refs.width_ref />
            </label>
            <label for = "height">
                "Height (px):"
                <input name = "height" id = "height" value = 800 type = "number" node_ref = display_settings_node_refs.height_ref />
            </label>
        </Section>
    }
}
