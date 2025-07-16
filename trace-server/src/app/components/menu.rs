use leptos::{component, view, IntoView, prelude::*};

use crate::app::components::Panel;

#[component]
pub(crate) fn Menu() -> impl IntoView {
    view! {
        <div class = "left">
            <h1>"Trace Viewer"</h1>
            <div class = "menu">
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </div>
        </div>
    }
}