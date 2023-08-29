use browser_rpass::request::*;
use browser_rpass::response::{GetResponse, ResponseEnum, ResponseEnumTrait};
use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use util::*;
use std::sync::Mutex;
use wasm_bindgen::convert::IntoWasmAbi;
use wasm_bindgen::prelude::*;

#[macro_use]
mod util;

mod api;
mod app;
mod components;
mod store;
use browser_rpass::store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP;

pub fn create_request_callback(
    request: RequestEnum,
    ctx: HashMap<String, String>,
    extension_port: Port,
) -> Box<impl FnOnce(ResponseEnum, Port)> {
    let on_message_cb = move |response: ResponseEnum, native_port: Port| {
        log!("response: {:?}", response);
    };
    Box::new(on_message_cb)
}
#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    let port = chrome.runtime().connect("binlemkoadegmiinejciimieplcjkkfo");
    let get_password_request = RequestEnum::create_get_request(
        "some.website.com".to_owned(),
        Resource::Password,
        Some(create_request_acknowledgement()),
        None,
    );
    let get_username_request = RequestEnum::create_search_request(
        "some.website.com".to_owned(),
        Resource::Username,
        Some(create_request_acknowledgement()),
        None,
    );
    let port2 = port.clone();
    let on_message_cb =
        Closure::<dyn Fn(String)>::new(move |msg: String| {
            match serde_json::from_str::<ResponseEnum>(&msg) {
                Ok(parsed_response) => {
                    let acknowledgement = parsed_response.get_acknowledgement().unwrap();
                    let request_cb = MESSAGE_ACKNOWLEDGEMENTS_POP_UP
                        .lock()
                        .unwrap()
                        .remove(&acknowledgement)
                        .unwrap();
                    request_cb(parsed_response, port2.clone());
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
    port.on_message()
        .add_listener(on_message_cb.as_ref().clone());
    on_message_cb.forget();
    let ctx = HashMap::new();
    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
        get_password_request.get_acknowledgement().clone().unwrap(),
        create_request_callback(get_password_request.clone(), ctx, port.clone()),
    );
    let ctx = HashMap::new();
    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
        get_username_request.get_acknowledgement().clone().unwrap(),
        create_request_callback(get_username_request.clone(), ctx, port.clone()),
    );
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&get_password_request).unwrap());
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&get_username_request).unwrap());
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
