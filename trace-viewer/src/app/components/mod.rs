mod setup;
mod results;
mod statusbar;
mod display;
mod menu;
mod broker;

pub(crate) use setup::Setup;
pub(crate) use results::Results;
pub(crate) use statusbar::Status;
pub(crate) use display::Display;
pub(crate) use menu::Menu;
pub(crate) use broker::BrokerSetup;

use leptos::{component, view, IntoView, prelude::*};

#[component]
pub(crate) fn Main() -> impl IntoView {
    view! {
        <div class = "middle">
        <BrokerSetup />
        <Setup />
        <Results />
        </div>
    }
}

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