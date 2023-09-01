use browser_rpass::request::*;
use browser_rpass::response::{GetResponse, ResponseEnum, ResponseEnumTrait};
use browser_rpass::util::*;
use browser_rpass::util::*;
use event_handlers::request_handlers::{create_response_listener, create_response_process_cb};
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Map;
use std::collections::HashMap;
use std::panic;
use std::sync::Mutex;
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::prelude::*;

mod api;
mod app;
mod components;
mod event_handlers;
mod store;
use browser_rpass::log;
use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_POP_UP};

#[wasm_bindgen(start)]
pub async fn run_app() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut passphrase_entry = HashMap::new();

    passphrase_entry.insert("passphrase".to_owned(), "abcd".to_owned());
    let json = JsValue::from_serde(&passphrase_entry).unwrap();
    // chrome.storage().session().set(json.clone()).await;

    let port = chrome.runtime().connect();
    // let mut header = HashMap::new();
    // header.insert("passphrase".to_owned(), passphrase);
    let get_password_request = RequestEnum::create_get_request(
        "some.website.com".to_owned(),
        Resource::Password,
        Some(create_request_acknowledgement()),
        None,
        // Some(header.clone()),
    );
    let get_username_request = RequestEnum::create_search_request(
        "some.website.com".to_owned(),
        Resource::Username,
        Some(create_request_acknowledgement()),
        None,
        // Some(header.clone()),
    );
    let on_message_cb = create_response_listener(port.clone());
    port.on_message()
        .add_listener(on_message_cb.as_ref().clone());
    on_message_cb.forget();
    let ctx = HashMap::new();
    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
        get_password_request.get_acknowledgement().clone().unwrap(),
        create_response_process_cb(get_password_request.clone(), ctx),
    );
    let ctx = HashMap::new();
    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
        get_username_request.get_acknowledgement().clone().unwrap(),
        create_response_process_cb(get_username_request.clone(), ctx),
    );
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&get_password_request).unwrap());
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&get_username_request).unwrap());
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
