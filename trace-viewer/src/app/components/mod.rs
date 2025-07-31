mod control_box;
mod topbar;
mod section;

pub(crate) use control_box::{ControlBoxWithLabel, InputBoxWithLabel, SubmitBox};
pub(crate) use topbar::TopBar;

use leptos::{IntoView, component, prelude::*, view};

fn build_classes_string(main: &'static str, mut classes: Vec<&'static str>) -> String {
    classes.push(main);
    classes
        .into_iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub(crate) fn Section(
    name: &'static str,
    #[prop(optional)] classes: Vec<&'static str>,
    children: Children,
) -> impl IntoView {
    let class = build_classes_string("section", classes);

    view! {
        <div class = "section-container">
            <div class = "section-name">
                {name}
            </div>
            <div class = class>
                {children()}
            </div>
        </div>
    }
}

#[component]
pub(crate) fn Panel(
    #[prop(optional)] classes: Vec<&'static str>,
    children: Children,
) -> impl IntoView {
    let class = build_classes_string("panel", classes);

    view! {
        <div class = class>
            {children()}
        </div>
    }
}
