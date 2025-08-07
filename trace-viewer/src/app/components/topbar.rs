use leptos::{IntoView, component, prelude::*, view};

use crate::app::components::Panel;

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    view! {
        <div class = "topbar">
            <h1>"Trace Viewer"</h1>
            <Panel classes = vec!["menu"]>
                <a href = "/"><div>Search</div></a>
                <a href = "/help"><div>Help</div></a>
            </Panel>
        </div>
    }
}
