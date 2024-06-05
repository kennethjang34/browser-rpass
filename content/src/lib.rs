mod event_handlers;
use std::panic;
mod store;

use gloo_utils::document;
use wasm_bindgen::prelude::*;
#[macro_use]
mod util;
mod app;
mod components;
pub use browser_rpass::{dbg, DataFieldType};
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
    let _password_suggestion_handle =
        yew::Renderer::<app::App>::with_root_and_props(root.into(), app::Props {}).render();
}
