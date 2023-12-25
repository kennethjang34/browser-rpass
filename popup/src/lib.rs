#[macro_use]
pub extern crate browser_rpass;
pub use browser_rpass::dbg;
use browser_rpass::js_binding::extension_api::chrome;
use browser_rpass::js_binding::extension_api::Tab;
pub use browser_rpass::types::*;
use gloo_utils::document;
use serde_json::json;
use serde_json::Value;
use store::PopupStore;
use store::EXTENSION_PORT;
use wasm_bindgen::JsValue;

use cfg_if::cfg_if;
pub use gloo_utils::format::JsValueSerdeExt;
#[warn(unused_imports)]
use log::{self, *};
use std::panic;
use std::rc::Rc;
use yew::Reducible;
use yewdux::dispatch::Dispatch;

mod api;
mod app;
mod components;
mod event_handlers;
mod pages;
mod store;
use std::collections::HashMap;

use browser_rpass::{create_request_acknowledgement, response::RequestEnum};

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            let _=browser_rpass::setup_logger();
        }
    } else {
        fn init_log() {}
    }
}

pub async fn run_app() -> Result<(), JsValue> {
    init_log();
    trace!("run_app");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let tabs = wasm_bindgen_futures::JsFuture::from(
        chrome.tabs().query(
            <JsValue as JsValueSerdeExt>::from_serde(&json!({"active":true,"currentWindow":true}))
                .unwrap(),
        ),
    )
    .await?;
    let tabs: Value = <JsValue as JsValueSerdeExt>::into_serde(&tabs).unwrap();
    let tabs = tabs.as_array().unwrap();
    let tabs = tabs
        .into_iter()
        .map(|tab| {
            <JsValue as JsValueSerdeExt>::from_serde(tab)
                .unwrap()
                .into()
        })
        .collect::<Vec<Tab>>();
    let tab = tabs.get(0).unwrap();
    let window_id = tab.window_id();
    let persisted = PopupStore::load_window_storage(&window_id.to_string()).await;
    if let Some(parsed_state) = persisted {
        let mut popup_store = PopupStore::default();
        popup_store.persistent_data = parsed_state;

        let dark_mode = popup_store.persistent_data.dark_mode;
        if dark_mode {
            let _ = document().body().unwrap().set_class_name("dark");
        } else {
            let _ = document().body().unwrap().class_list().remove_1("dark");
        }
        popup_store.window_id = Some(window_id.to_string());
        Dispatch::<PopupStore>::new().set(popup_store);
    } else {
        Dispatch::<PopupStore>::new().set(PopupStore {
            window_id: Some(window_id.to_string()),
            ..Default::default()
        })
    }
    let acknowledgement = create_request_acknowledgement();
    let init_config = HashMap::new();
    let init_request =
        RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());

    yew::Renderer::<app::App>::new().render();
    Ok(())
}

#[derive(Clone, Debug, Default, Copy)]
pub struct BoolState {
    value: bool,
}
impl From<&BoolState> for bool {
    fn from(state: &BoolState) -> bool {
        state.value
    }
}
impl From<BoolState> for bool {
    fn from(state: BoolState) -> bool {
        state.value
    }
}
#[derive(Clone, Debug)]
pub enum BoolStateAction {
    ToggleAction,
    SetAction(bool),
}

impl Reducible for BoolState {
    type Action = BoolStateAction;
    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let next_value = match action {
            BoolStateAction::ToggleAction => !self.value,
            BoolStateAction::SetAction(value) => value,
        };
        Self { value: next_value }.into()
    }
}
