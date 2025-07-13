mod setup;
mod broker_info;
mod results;
mod statusbar;
mod display;

pub(crate) use setup::{Controls, Setup};
pub(crate) use broker_info::BrokerInfo;
pub(crate) use results::Results;
pub(crate) use statusbar::Status;
pub(crate) use display::Display;

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
pub(crate) fn Panel(name: &'static str, children: Children) -> impl IntoView {
    view!{
        <div class = "panel">
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
pub(crate) fn Main() -> impl IntoView {
    view! {
        <Setup />
        <Controls />
        <Status />
        <Section name = "Results">
        <Results />
        <Display />
        </Section>
    }
}