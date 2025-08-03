pub(crate) mod components;
mod main_page;
pub(crate) mod sections;

use crate::{
    app::components::TopBar,
    structs::{Select, Topics},
};
use leptos::{prelude::*, server_fn::codec::IntoReq};
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use main_page::Main;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AppUuid {
    pub inner: String,
}

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
    provide_context(default);

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
                <App/>
            </body>
        </html>
    }
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // sets the document title
        <Title text="Trace Viewer" />

        <Stylesheet href="/pkg/TraceViewer.css"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <TopBar />
        <Router>
            <Routes fallback=|| view! { }>
                <Route path=path!("/") view=Main />
            </Routes>
        </Router>
    }
}
