use leptos::{component, view, IntoView, prelude::*};

use crate::app::components::Panel;

#[component]
pub(crate) fn Display() -> impl IntoView {
    view! {
        <Panel>
            "No Display"
        </Panel>
    }
}