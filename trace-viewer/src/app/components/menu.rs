use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn Menu() -> impl IntoView {
    view! {
        <div class = "left-column">
            <h1>"Trace Viewer"</h1>
            <div class = "menu">
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </div>
        </div>
    }
}
