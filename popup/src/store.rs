use browser_rpass::log;
use browser_rpass::request::RequestEnum;
use browser_rpass::store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP;
use browser_rpass::util::{chrome, create_request_acknowledgement, Port};
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{self, Rc};
use wasm_bindgen::JsValue;

lazy_static! {
    pub static ref EXTENSION_PORT: Port = chrome.runtime().connect();
}
use yew::prelude::*;
use yewdux::storage::{self, Area};
use yewdux::{dispatch, prelude::*};

use crate::api::types::{Account, User};
use crate::event_handlers::request_handlers::create_response_process_cb;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum StateStatus {
    #[default]
    Idle,
    Loading,
    Success,
    Failure,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct PopupStore {
    pub page_loading: bool,
    pub alert_input: AlertInput,
    pub verified: bool,
    pub status: StateStatus,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoginAction {
    LoginStarted,
    LoginError,
    LoginSucceeded,
    LoginFailed,
    LogoutSucceeded,
    LogoutFailed,
    LogoutStarted,
    Logout,
    Login,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum PopupAction {
    Login(LoginAction),
}
impl Reducer<PopupStore> for LoginAction {
    fn apply(self, store: Rc<PopupStore>) -> Rc<PopupStore> {
        match self {
            LoginAction::LoginStarted => PopupStore {
                page_loading: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginError => PopupStore {
                page_loading: false,
                status: StateStatus::Error,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginSucceeded => PopupStore {
                page_loading: false,
                verified: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginFailed => PopupStore {
                page_loading: false,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutSucceeded => PopupStore {
                page_loading: false,
                verified: false,
                status: StateStatus::Success,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutFailed => PopupStore {
                page_loading: false,
                status: StateStatus::Failure,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutStarted => PopupStore {
                page_loading: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Logout => PopupStore {
                verified: false,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Login => PopupStore {
                verified: true,
                ..store.deref().clone()
            }
            .into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AlertInput {
    pub show_alert: bool,
    pub alert_message: String,
}

impl Store for PopupStore {
    fn new() -> Self {
        init_listener(StorageListener);
        PopupStore::load();
        let mut popup_store = PopupStore::default();
        popup_store
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl PopupStore {
    pub fn load() {
        let ctx = HashMap::new();
        let acknowledgement = create_request_acknowledgement();
        let mut init_config = HashMap::new();
        let init_request =
            RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
        MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
            acknowledgement.clone(),
            create_response_process_cb(init_request.clone(), ctx),
        );
        EXTENSION_PORT
            .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = PopupStore;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        log!("on_change called for PopupStore");
        log!("{}", format!("popup store new state: {:?}", state));
    }
}
