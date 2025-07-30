pub(crate) mod components;
pub(crate) mod sections;
mod main_page;

use leptos::prelude::*;
use leptos_meta::*;

use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    app::components::Menu,
    structs::{Select, Topics},
};
use main_page::Main;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DefaultData {
    pub broker: String,
    pub topics: Topics,
    pub select: Select,
    pub username: Option<String>,
    pub password: Option<String>,
    pub consumer_group: String,
    pub poll_broker_timeout_ms: u64,
}

pub fn shell(leptos_options: LeptosOptions, default: DefaultData) -> impl IntoView + 'static {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=leptos_options.clone() />
                <HydrationScripts options=leptos_options.clone()/>
                <MetaTags/>
            </head>
            <body>
                <App default/>
            </body>
        </html>
    }
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App(default: DefaultData) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    provide_context(default);
    if let Err(e) = leptos_server_signal::provide_websocket("ws://localhost:3000/ws") {
        if let Some(e) = e.as_string() {
            warn!("Could not provide websockets to client: {e}");
        } else {
            warn!("Could not provide websockets to client: (Error could not be parsed)");
        }
    }

    view! {
        // sets the document title
        <Title text="Trace Viewer" />

        <Stylesheet href="/pkg/TraceViewer.css"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Menu />
        <Router>
            <Routes fallback=|| view! { }>
                <Route path=path!("/") view=Main />
            </Routes>
        </Router>
    }
}