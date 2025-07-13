use leptos::{component, view, IntoView, prelude::*};

use crate::web::components::Panel;

#[component]
pub(crate) fn Display() -> impl IntoView {
    view! {
        <Panel name = "Display">
            "No Display"
        </Panel>
    }
}