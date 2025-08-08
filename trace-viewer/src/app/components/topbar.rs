use leptos::{IntoView, component, prelude::*, view};

use crate::structs::ClientSideData;

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    let client_side_data = use_context::<ClientSideData>()
        .expect("ClientSideData should be provided, this should never fail.");
    view! {
        <div class = "topbar">
            <div class = "title">"Trace Viewer"</div>
            <div class = "subtitle">{client_side_data.broker_name}</div>
            <div class = "menu">
                <a href = "/"><span>Home</span></a>
                <Show when = {let link_to_redpanda_console = client_side_data.link_to_redpanda_console.clone(); move ||link_to_redpanda_console.is_some()}>
                    <a href = {let link_to_redpanda_console = client_side_data.link_to_redpanda_console.clone(); link_to_redpanda_console}><span>Redpanda Console</span></a>
                </Show>
                <a href = "/help"><span>Help</span></a>
            </div>
        </div>
    }
}
