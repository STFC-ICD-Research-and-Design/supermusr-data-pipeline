
use leptos::{attr::AttributeValue, component, ev::{Event, Targeted}, html::{HtmlElement, Input}, prelude::*, view, IntoView};

use crate::app::components::build_classes_string;

#[component]
pub(crate) fn InputBoxWithLabel(
    name: &'static str,
    label: &'static str,
    input_type: &'static str,
    value: impl AttributeValue,
    #[prop(optional)]
    node_ref: NodeRef<Input>
) -> impl IntoView {
    view!{
        <label class = "panel-item" for = {name}>
            {label}
        </label>
        <input class = "panel-item" name = name value = value type = input_type node_ref = node_ref />
    }
}

#[component]
pub(crate) fn ControlBoxWithLabel(
    name: &'static str,
    label: &'static str,
    children: Children
) -> impl IntoView {
    view!{
        <label class = "panel-item" for = {name}>
            {label}
        </label>
        {children()}
    }
}

#[component]
pub(crate) fn SubmitBox(
    label: &'static str,
    #[prop(optional)]
    classes: Vec<&'static str>
) -> impl IntoView {
    let class = build_classes_string("panel-item across-two-cols", classes);
    view!{
        <input type = "submit" class = class value = label />
    }
}