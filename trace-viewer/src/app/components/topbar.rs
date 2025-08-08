use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    view! {
        <div class = "topbar">
            <div class = "title">"Trace Viewer"</div>
            <div class = "menu">
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </div>
        </div>
    }
}
