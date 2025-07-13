use anyhow as _;
use chrono as _;
use clap as _;
use serde as _;
use serde_json as _;
use tracing_subscriber as _;
use leptos::prelude::*;
use trace_server::App;
use leptos_meta as _;
use leptos_router as _;

use trace_server::finder::SearchEngine;

fn main() {
    // set up logging
    //_ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let finder = SearchEngine::new();

    mount_to_body(|| {
        view! {
            <App finder = finder />
        }
    })
}