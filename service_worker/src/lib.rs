use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
#[macro_use]
mod util;

use util::*;

#[wasm_bindgen(start)]
pub async fn main() {
    let native_port = chrome.runtime().connect_native("com.rpass");
    let on_native_message_cb = Closure::<dyn Fn(String)>::new(move |msg: String| {
        log!("msg from native app : {:?}", msg);
    });
    native_port
        .on_message()
        .add_listener(on_native_message_cb.as_ref().clone());
    on_native_message_cb.forget();

    let mut hashmap = HashMap::new();
    hashmap.insert("text".to_owned(), "testing messaging".to_owned());
    native_port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&hashmap).unwrap());

    let on_connect_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                log!("message received at service worker: {:?}", msg);
            }
        });
        port.on_message().add_listener(cb.into_js_value());
        port.post_message("hello from service worker".into());
    });

    chrome
        .runtime()
        .on_connect()
        .add_listener(on_connect_cb.into_js_value());
}
