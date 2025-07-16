#![allow(unused_imports)]

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_meta::*;

use miette as _;
use thiserror as _;
use tracing as _;

#[cfg(feature = "ssr")]
use rdkafka as _;
#[cfg(feature = "ssr")]
use console_error_panic_hook as _;
#[cfg(feature = "ssr")]
use tokio as _;
#[cfg(feature = "ssr")]
use thiserror as _;
#[cfg(feature = "ssr")]
use supermusr_streaming_types as _;
#[cfg(feature = "ssr")]
use strum as _;
#[cfg(feature = "ssr")]
use tracing as _;

//use tachys::view::add_attr::AddAnyAttr;

// Modules
//mod components;
//mod pages;

// Top-Level pages
//use crate::pages::home::Home;

pub mod cli_structs;
mod web;

#[cfg(feature = "ssr")]
mod messages;
#[cfg(feature = "ssr")]
pub mod finder;

use web::components::Main;

pub type Timestamp = DateTime<Utc>;

use cli_structs::{Topics, Select};
pub struct DefaultData {
    pub broker: String,
    pub topics : Topics,
    pub select: Select,
}

/// An app router which renders the homepage and handles 404's
#[component]
pub fn App(default: DefaultData) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html />

        // sets the document title
        <Title text="Welcome to Leptos CSR" />

        // injects metadata in the <head> of the page
        <Meta charset="UTF-8" />
        <Meta name="viewport" content="width=device-width, initial-scale=1.0" />

        <Main default = default />
        /*<Router>
            <Routes fallback=|| view! { }>
                <Route path=path!("/") view=Main />
            </Routes>
        </Router>*/
    }
}