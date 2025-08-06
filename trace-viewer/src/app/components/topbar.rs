use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    view! {
        <div class = "topbar">
            <h1>"Trace Viewer"</h1>
            <div class = "menu">
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </div>
        </div>
    }
}
