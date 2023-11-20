pub use browser_rpass;
pub use browser_rpass::request;
pub use browser_rpass::request::RequestEnum;
pub use browser_rpass::util::*;
use event_handlers::native_message_handler::*;
use event_handlers::popup_request_handler::*;
pub use gloo_utils::format::JsValueSerdeExt;
use std::panic;
pub use store::NATIVE_PORT;
use wasm_bindgen::prelude::wasm_bindgen;
mod store;
pub use browser_rpass::types::*;
use cfg_if::cfg_if;
pub use log::*;

pub use browser_rpass::dbg;
mod api;

mod event_handlers;

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

    chrome
        .runtime()
        .on_connect()
        .add_listener(create_request_listener().into_js_value());
}
