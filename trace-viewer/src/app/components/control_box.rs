
use leptos::{component, view, IntoView, prelude::*};

#[component]
pub(crate) fn ControlBoxWithLabel(name: &'static str, label: &'static str, children: Children) -> impl IntoView {
    view!{
        <div class = "control-box">
            <label for = {name}>
                {label}
            </label>
            {children()}
        </div>
    }
}

#[component]
pub(crate) fn ControlBox(children: Children) -> impl IntoView {
    view!{
        <div class = "control-box">
            {children()}
        </div>
    }
}