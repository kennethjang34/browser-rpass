#[macro_use]
pub extern crate browser_rpass;
pub use browser_rpass::dbg;
pub use browser_rpass::types::*;
use gloo_utils::document;
use store::PopupStore;
use store::EXTENSION_PORT;
use wasm_bindgen::JsValue;

use cfg_if::cfg_if;
pub use gloo_utils::format::JsValueSerdeExt;
#[warn(unused_imports)]
use log::{self, *};
use std::panic;
use yewdux::dispatch::Dispatch;

mod api;
mod app;
mod components;
mod event_handlers;
mod pages;
mod store;
use std::collections::HashMap;

use browser_rpass::{create_request_acknowledgement, response::RequestEnum};
cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            let _=browser_rpass::setup_logger();
        }
    } else {
        fn init_log() {}
    }
}

pub async fn run_app() -> Result<(), JsValue> {
    trace!("run_app");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_log();
    let persisted = PopupStore::load().await;
    if let Some(parsed_state) = persisted {
        let mut popup_store = PopupStore::default();
        popup_store.persistent_data = parsed_state;
        let dark_mode = popup_store.persistent_data.dark_mode;
        if dark_mode {
            let _ = document().body().unwrap().set_class_name("dark");
        } else {
            let _ = document().body().unwrap().class_list().remove_1("dark");
        }
        Dispatch::<PopupStore>::new().set(popup_store);
    }
    let acknowledgement = create_request_acknowledgement();
    let init_config = HashMap::new();
    let init_request =
        RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());

    yew::Renderer::<app::App>::new().render();
    Ok(())
}
