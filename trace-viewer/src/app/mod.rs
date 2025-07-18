pub(crate) mod components;
pub(crate) mod sections;

use leptos::prelude::*;
use leptos_meta::*;

use leptos_router::{components::{Route, Router, Routes}, path};
use serde::{Deserialize, Serialize};

use crate::{app::{components::Menu, sections::{BrokerSetup, Search}}, structs::{Select, Topics}};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct DefaultData {
    pub broker: String,
    pub topics : Topics,
    pub select: Select,
    pub username: Option<String>,
    pub password: Option<String>,
    pub consumer_group: String,
    pub poll_broker_timeout_ms: u64,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PollBrokerData {
    pub broker: Option<String>,
    pub topics : Option<Topics>,
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

#[component]
pub(crate) fn Main() -> impl IntoView {
    view! {
        <div class = "middle">
            <BrokerSetup />
            <Search />
        </div>
    }
}