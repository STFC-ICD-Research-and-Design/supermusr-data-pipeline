use leptos::{IntoView, attr::AttributeValue, component, html::Input, prelude::*, view};

use crate::app::components::build_classes_string;

#[component]
pub(crate) fn InputBoxWithLabel(
    name: &'static str,
    label: &'static str,
    input_type: &'static str,
    value: impl AttributeValue,
    #[prop(optional)] node_ref: NodeRef<Input>,
) -> impl IntoView {
    view! {
        <label class = "panel-item" for = {name}>
            {label}
            <input class = "panel-item" name = name id = name value = value type = input_type node_ref = node_ref />
        </label>
    }
}

#[component]
pub(crate) fn SubmitBox(
    label: &'static str,
    #[prop(optional)] classes: Vec<&'static str>,
) -> impl IntoView {
    let class = build_classes_string("panel-item across-two-cols", classes);
    view! {
        <input type = "submit" class = class value = label />
    }
}
