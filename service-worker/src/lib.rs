use event_handlers::request_handlers::*;
use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use std::panic;
use store::NATIVE_PORT;
use wasm_bindgen::prelude::*;

use browser_rpass::log;
use browser_rpass::request::*;
use browser_rpass::util::*;
mod store;

#[cfg(test)]
mod tests;

mod event_handlers;

use browser_rpass::store::MESSAGE_ACKNOWLEDGEMENTS_NATIVE;
#[wasm_bindgen(start)]
pub async fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let chrome_storage = chrome.storage().session();
    let on_native_message_cb = Closure::<dyn Fn(String)>::new(move |msg: String| {
        match serde_json::from_slice::<serde_json::Value>(&msg.as_bytes()) {
            Ok(parsed_response) => {
                let acknowledgement = parsed_response
                    .get("acknowledgement")
                    .unwrap()
                    .as_str()
                    .unwrap();
                let response_cb = MESSAGE_ACKNOWLEDGEMENTS_NATIVE
                    .lock()
                    .unwrap()
                    .remove(acknowledgement)
                    .unwrap();
                response_cb(&msg.as_bytes(), NATIVE_PORT.clone());
            }
            Err(e) => {
                log!(
                    "error happend while parsing:{:?}. Error message: {:?}",
                    msg,
                    e
                );
            }
        }
    });
    let on_native_port_disconnect_cb = Closure::<dyn Fn(Port)>::new(move |port| {
        log!("native port disconnected");
        log!("port: {:?}", port);
    });
    NATIVE_PORT
        .on_disconnect()
        .add_listener(on_native_port_disconnect_cb.as_ref().clone());
    NATIVE_PORT
        .on_message()
        .add_listener(on_native_message_cb.as_ref().clone());
    on_native_message_cb.forget();
    on_native_port_disconnect_cb.forget();

    let mut init_config = HashMap::new();
    init_config.insert("home_dir".to_owned(), "/Users/JANG".to_owned());
    let init_request = RequestEnum::create_init_request(init_config, None, None);
    NATIVE_PORT.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());

    chrome
        .runtime()
        .on_connect()
        .add_listener(create_request_listener().into_js_value());
}
