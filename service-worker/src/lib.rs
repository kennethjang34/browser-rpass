use event_handlers::request_handlers::*;
use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
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
    NATIVE_PORT
        .on_message()
        .add_listener(on_native_message_cb.as_ref().clone());
    on_native_message_cb.forget();

    let mut init_config = HashMap::new();
    init_config.insert("home_dir".to_owned(), "/Users/JANG".to_owned());
    let init_request = RequestEnum::create_init_request(init_config, None, None);
    NATIVE_PORT.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());

    // let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
    //     let native_port = native_port.clone();
    //     let cb = Closure::<dyn Fn(JsValue, Port)>::new({
    //         move |msg: JsValue, port: Port| {
    //             let request: RequestEnum = <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
    //             handle_request_from_popup(request, &port, &native_port).unwrap();
    //         }
    //     });
    //     port.on_message().add_listener(cb.into_js_value());
    // });
    // on_connect_with_popup_cb(extension_port, native_port)
    chrome
        .runtime()
        .on_connect()
        .add_listener(create_request_listener().into_js_value());
}
