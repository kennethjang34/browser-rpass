#[macro_use]
pub extern crate browser_rpass;
pub use browser_rpass::dbg;
pub use browser_rpass::types::*;

use cfg_if::cfg_if;
pub use gloo_utils::format::JsValueSerdeExt;
use log::{self, *};
use std::panic;
use wasm_bindgen::prelude::*;

mod api;
mod app;
mod components;
mod event_handlers;
mod pages;
mod router;
mod store;
cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            console_log::init_with_level(Level::Trace).expect("error initializing log");
        }
    } else {
        fn init_log() {}
    }
}

#[wasm_bindgen(start)]
pub async fn run_app() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_log();

    yew::Renderer::<app::App>::new().render();
    Ok(())
}
