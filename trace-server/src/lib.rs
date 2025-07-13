use anyhow as _;
use serde as _;
use serde_json as _;
use tracing_subscriber as _;
use chrono::{DateTime, Utc};
use console_error_panic_hook as _;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

// Modules
//mod components;
//mod pages;

// Top-Level pages
//use crate::pages::home::Home;

mod web;
mod messages;
pub mod finder;
pub mod cli_structs;

use web::components::Main;
use finder::MessageFinder;
use cli_structs::Topics;

type Timestamp = DateTime<Utc>;

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App<Finder: MessageFinder>(finder : Finder) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html attr:lang="en" attr:dir="ltr" attr:data-theme="light" />

        // sets the document title
        <Title text="Welcome to Leptos CSR" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Main finder = finder />
        /*<Router>
            <Routes fallback=|| view! { }>
                <Route path=path!("/") view=Main />
            </Routes>
        </Router>*/
    }
}