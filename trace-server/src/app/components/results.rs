use leptos::{component, view, IntoView, prelude::*};

use crate::app::components::Panel;

#[component]
pub(crate) fn Results() -> impl IntoView {
    view! {
        <Panel name = "Search Results">
            "No Results"
        </Panel>
    }
}