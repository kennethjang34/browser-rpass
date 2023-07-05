use reqwest_wasm::get;
use std::f32::consts::E;
use wasm_bindgen::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

#[macro_use]
mod util;
mod api;
mod app;
mod components;
mod store;

#[wasm_bindgen(start)]
pub fn run_app() -> Result<(), JsValue> {
    yew::Renderer::<app::App>::new().render();
    Ok(())
}
