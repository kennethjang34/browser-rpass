use crate::store::PopupStore;
#[allow(warnings)]
pub(crate) use browser_rpass::request::*;
pub(crate) use browser_rpass::response::{GetResponse, ResponseEnum, ResponseEnumTrait};
pub(crate) use browser_rpass::util::*;
pub(crate) use browser_rpass::util::*;
pub(crate) use event_handlers::request_handlers::{
    create_response_listener, create_response_process_cb,
};
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
mod pages;
mod router;
mod store;
pub(crate) use browser_rpass::log;
pub(crate) use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_POP_UP};

#[wasm_bindgen(start)]
pub async fn run_app() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
