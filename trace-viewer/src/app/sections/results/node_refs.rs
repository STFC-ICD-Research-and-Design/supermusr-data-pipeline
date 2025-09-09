use leptos::{
    html::{Input, Select},
    prelude::*,
};

#[derive(Default, Clone, Copy)]
pub(crate) struct ResultsSettingsNodeRefs {
    pub(crate) display_all_channels: NodeRef<Input>,
    pub(crate) display_mode: NodeRef<Select>,
}
