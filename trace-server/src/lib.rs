#![allow(unused_crate_dependencies)]

mod web;
mod messages;
pub mod structs;

use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
pub use web::{App, DefaultData, shell};

pub type Timestamp = DateTime<Utc>;
pub type Channel = u32;
pub type Time = u32;
pub type Intensity = u16;
pub type DigitizerId = u8;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod finder;
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}