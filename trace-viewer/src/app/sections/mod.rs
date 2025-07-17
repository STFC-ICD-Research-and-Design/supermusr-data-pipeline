mod search;
mod results;
mod statusbar;
mod display;
mod broker;

pub(crate) use search::Search;
pub(crate) use results::Results;
pub(crate) use display::Display;
pub(crate) use broker::BrokerSetup;

use leptos::{component, view, IntoView, prelude::*};

#[component]
pub(crate) fn Main() -> impl IntoView {
    view! {
        <div class = "middle">
        <BrokerSetup />
        <Search />
        <Results />
        </div>
    }
}