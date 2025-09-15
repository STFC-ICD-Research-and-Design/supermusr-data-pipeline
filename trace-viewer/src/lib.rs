#![allow(unused_crate_dependencies)]
#![recursion_limit = "256"]

pub mod app;
pub mod structs;

pub use app::{App, shell};

use cfg_if::cfg_if;
use chrono::{DateTime, Utc};

/// Used by instances of the website to refer to server-side sessions.
pub type Uuid = Option<String>;
/// The timestamp type with timezone.
pub type Timestamp = DateTime<Utc>;

/*
    The following types are redefined manually rather than being imported
    from [common] as that module cannot be included in the wasm target.
    Maybe this can be fixed later?
*/
pub type Channel = u32;
pub type Time = u32;
pub type Intensity = u16;
pub type DigitizerId = u8;
pub type FrameNumber = u32;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod sessions;
        pub mod finder;
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    leptos::mount::hydrate_body(App);
}
