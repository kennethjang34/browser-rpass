use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use util::*;
use wasm_bindgen::prelude::*;

#[macro_use]
mod util;

mod api;
mod app;
mod components;
mod store;

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    let mut hashmap = HashMap::new();
    hashmap.insert("text".to_owned(), "dassd".to_owned());
    let port = chrome.runtime().connect("binlemkoadegmiinejciimieplcjkkfo");
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&hashmap).unwrap());
    let callback = Closure::<dyn Fn(JsValue, Port)>::new(move |msg: JsValue, port: Port| {
        log!("message received at popup: {:?}", msg);
        port.post_message(
            <JsValue as JsValueSerdeExt>::from_serde(&format!(
                "hello from popup! I have received message {:?}",
                msg
            ))
            .unwrap(),
        );
    });
    port.on_message().add_listener(callback.into_js_value());
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
