#![allow(unused_crate_dependencies)]

pub mod app;
mod messages;
pub mod structs;

use cfg_if::cfg_if;
use chrono::{DateTime, Utc};
pub use app::{App, DefaultData, shell};

pub type Timestamp = DateTime<Utc>;
pub type Channel = u32;
pub type Time = u32;
pub type Intensity = u16;
pub type DigitizerId = u8;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        pub mod finder;
        //pub mod graphics;
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(||App(app::AppProps { default: DefaultData::default() }));
}