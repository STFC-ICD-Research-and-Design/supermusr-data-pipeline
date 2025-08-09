pub(crate) mod components;
mod main_content;
pub(crate) mod sections;
pub(crate) mod server_functions;

use crate::{
    app::{components::TopBar, server_functions::get_client_side_data},
    structs::{ClientSideData, DefaultData},
};
use leptos::{logging, prelude::*};
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path, SsrMode,
};
use main_content::Main;

pub(crate) type Uuid = Option<String>;

pub fn shell(leptos_options: LeptosOptions, client_side_data: ClientSideData) -> impl IntoView + 'static {
    provide_context(client_side_data);

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <script src="https://cdn.plot.ly/plotly-2.14.0.min.js"></script>
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
    let csd = SharedValue::new(||{
        use_context::<ClientSideData>()
            .expect("Client-side data should be provided, this should never fail.")
    });
    provide_context::<ClientSideData>(csd.into_inner());

    view! {
        // sets the document title
        <Title text="Trace Viewer" />

        <Stylesheet href="/pkg/TraceViewer.css"/>

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <TopBar />
        <Router>
            <Routes fallback=|| ()>
                <Route path=path!("/") view=Main/>
            </Routes>
        </Router>
    }
}
