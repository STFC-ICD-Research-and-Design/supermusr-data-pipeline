mod menu;
mod control_box;

pub(crate) use menu::Menu;
pub(crate) use control_box::{ControlBox, ControlBoxWithLabel};

use leptos::{component, view, IntoView, prelude::*};

#[component]
pub(crate) fn Section(name: &'static str, children: Children) -> impl IntoView {
    view!{
        <div class = "section">
            <div class = "name">
                {name}
            </div>
            <div class = "content">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub(crate) fn Panel(children: Children) -> impl IntoView {
    view!{
        <div class = "panel">
            {children()}
        </div>
    }
}

#[component]
pub(crate) fn VerticalBlock(children: Children) -> impl IntoView {
    view!{
        <div class = "block">
            {children()}
        </div>
    }
}