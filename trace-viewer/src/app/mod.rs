pub(crate) mod components;
pub(crate) mod sections;

use leptos::prelude::*;
use leptos_meta::*;

use sections::Main;
use leptos_router::{components::{Route, Router, Routes}, path};


use crate::{app::components::Menu, structs::{Select, Topics}};

#[derive(Default, Clone)]
pub struct DefaultData {
    pub broker: String,
    pub topics : Topics,
    pub select: Select,
}

pub fn shell(leptos_options: LeptosOptions) -> impl IntoView + 'static {
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

        <Menu />
        <Router>
            <Routes fallback=|| view! { }>
                <Route path=path!("/") view=Main />
            </Routes>
        </Router>
    }
}