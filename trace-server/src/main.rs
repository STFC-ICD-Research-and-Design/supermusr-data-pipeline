use leptos::prelude::*;
use trace_server::App;
use leptos_meta as _;
use leptos_router as _;

fn main() {
    // set up logging
    //_ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <App />
        }
    })
}