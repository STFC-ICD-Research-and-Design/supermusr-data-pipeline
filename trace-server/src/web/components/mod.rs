mod setup;
mod broker_info;
mod results;
mod statusbar;
mod display;

pub(crate) use setup::Setup;
pub(crate) use broker_info::BrokerInfo;
pub(crate) use results::Results;
pub(crate) use statusbar::Status;
pub(crate) use display::Display;

use leptos::{component, html::Div, view, IntoView, prelude::*};

#[component]
pub(crate) fn Main() -> impl IntoView {
    view! {
        <Setup />
        <Status />
        <Results />
        <Display />
    }
}