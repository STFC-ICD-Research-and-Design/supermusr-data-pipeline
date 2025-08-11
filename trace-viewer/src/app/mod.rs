mod components;
mod main_content;
mod sections;
mod server_functions;
mod topbar;

use cfg_if::cfg_if;
use crate::structs::ClientSideData;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path
};
use topbar::TopBar;
use main_content::Main;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub(crate) use server_functions::{ServerError, SessionError};
    }
}

/// This struct enable a degree of type-checking for the [use_context]/[use_context] functions.
/// Any component making use of the following fields should call `use_context::<TopLevelContext>()`
/// and select the desired field.
#[derive(Clone)]
pub(crate) struct TopLevelContext {
    client_side_data: ClientSideData
}

pub fn shell(leptos_options: LeptosOptions) -> impl IntoView + 'static {

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

    let client_side_data = SharedValue::new(||{
        use_context::<ClientSideData>()
            .expect("TopLevelContext should be provided, this should never fail.")
    })
    .into_inner();
    provide_context(TopLevelContext { client_side_data });

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
