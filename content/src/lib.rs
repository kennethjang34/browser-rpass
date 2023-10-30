mod event_handlers;
use std::panic;
mod store;

use browser_rpass::chrome;
use gloo_utils::{document, window};
use wasm_bindgen::prelude::*;
#[macro_use]
mod util;
mod app;
use app::*;
mod components;
pub use browser_rpass::dbg;
use cfg_if::cfg_if;
pub use log::*;

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            let _=browser_rpass::setup_logger();
        }
    } else {
        fn init_log() {}
    }
}

#[wasm_bindgen(start)]
pub async fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_log();
    let root = document().create_element("div").unwrap();
    document().body().unwrap().append_child(&root).unwrap();
    let _password_suggestion_handle = yew::Renderer::<App>::with_root_and_props(
        root.into(),
        Props {
            address: window().location().href().unwrap_or_default(),
        },
    )
    .render();
}
