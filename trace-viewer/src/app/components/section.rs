pub(crate) use super::build_classes_string;

use leptos::{IntoView, component, either::Either, prelude::*, view};
use tracing::warn;

#[component]
pub(crate) fn Section(
    name: &'static str,
    #[prop(optional, default = false)] closable: bool,
    #[prop(optional)] classes: Vec<&'static str>,
    children: Children,
) -> impl IntoView {
    let class = build_classes_string("section", classes);

    view! {
        <div class = "section-container">
            {
                if closable {
                    Either::Left(view!{
                        <SectionNameClosable name />
                    })
                } else {
                    Either::Right(view!{
                        <SectionNameUnclosable name />
                    })
                }
            }
            <div class = class>
                {children()}
            </div>
        </div>
    }
}

#[component]
pub(crate) fn SectionNameClosable(name: &'static str) -> impl IntoView {
    view! {
        <div class = "section-name" on:click:target = move |e| {
            if let Err(e) = e.target().parent_element().expect("").class_list().toggle("closed") {
                warn!("{}", e.as_string().unwrap());
            }
        }>
            {name}
        </div>
    }
}

#[component]
pub(crate) fn SectionNameUnclosable(name: &'static str) -> impl IntoView {
    view! {
        <div class = "section-name" on:click:target = move |e| {
            if let Err(e) = e.target().parent_element().expect("").class_list().toggle("closed") {
                warn!("{}", e.as_string().unwrap());
            }
        }>
            {name}
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
