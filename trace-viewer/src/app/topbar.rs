use leptos::{component, prelude::*, view, IntoView};

use crate::app::TopLevelContext;

#[component]
pub(crate) fn TopBar() -> impl IntoView {
    let client_side_data = use_context::<TopLevelContext>()
        .expect("ClientSideData should be provided, this should never fail.")
        .client_side_data;

    let red_panda_link = client_side_data.link_to_redpanda_console.map(|link|
        view!{<a href = {link.clone()}><span>Redpanda Console</span></a>}
    );
    let broker_name = client_side_data.broker_name;

    view! {
        <div class = "topbar">
            <div class = "title-box">
                <div class = "title">"Trace Viewer"</div>
                <div class = "subtitle">{broker_name}</div>
            </div>
            <div class = "menu">
                <a href = "/"><span>Home</span></a>
                {red_panda_link}
                <a href = "https://github.com/STFC-ICD-Research-and-Design/supermusr-data-pipeline/issues"><span>"Report an Issue"</span></a>
                <a href = "/help"><span>Help</span></a>
            </div>
        </div>
    }
}
