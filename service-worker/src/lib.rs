pub use browser_rpass;
pub use browser_rpass::js_binding::extension_api::chrome;
pub use browser_rpass::request;
pub use browser_rpass::request::RequestEnum;
use browser_rpass::request::SessionEvent;
use browser_rpass::request::SessionEventType;
use browser_rpass::response::MessageEnum;
pub use browser_rpass::util::*;
use event_handlers::native_message_handler::*;
use event_handlers::popup_request_handler::*;
pub use gloo_utils::format::JsValueSerdeExt;
use std::panic;
use store::SessionStore;
pub use store::NATIVE_PORT;
use wasm_bindgen::prelude::wasm_bindgen;
use yewdux::dispatch::Dispatch;
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
    trace!("service worker starts");

    //Currently load the session store from the extension session storage does not make any difference since
    //1. Service worker does not die out (extension message port will get created whenever the
    //   extension gets inactive, which will lead to the service worker to be activated again)
    //2. loading of session store happens only here, which runs only once when the service worker
    //   starts, which means the loaded state of the store is empty in session storage anyway
    //
    //However, we might want to persist the session store state in the future, so we keep this.
    //Because we are using session store, it should be safe to save data in it.
    let saved_state = SessionStore::load().await;
    if let Some(parsed_state) = saved_state {
        Dispatch::<SessionStore>::new().set(parsed_state);
    }

    chrome
        .runtime()
        .on_connect()
        .add_listener(create_request_listener().into_js_value());
}
