use browser_rpass::dbg;
use browser_rpass::request::SessionEventWrapper;
use browser_rpass::types::Account;
use browser_rpass::types::Resource;
use browser_rpass::types::StorageStatus;
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
use log::*;
use parking_lot::ReentrantMutex;
use serde_json;
use wasm_bindgen::prelude::Closure;
use yewdux::mrc::Mrc;

use crate::event_handlers::extension_message_listener::create_message_listener;
use browser_rpass::response::RequestEnum;
pub use browser_rpass::util::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::JsValue;

fn port_disconnect_handler(_port: Port) {
    let new_port = chrome.runtime().connect();
    new_port
        .on_disconnect()
        .add_listener(Closure::<dyn Fn(Port)>::new(port_disconnect_handler).into_js_value());
    new_port
        .on_message()
        .add_listener(create_message_listener(&new_port).into_js_value());
    let acknowledgement = create_request_acknowledgement();
    let init_config = HashMap::new();
    let init_request =
        RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
    new_port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
    *EXTENSION_PORT.lock().borrow_mut() = new_port;
}
lazy_static! {
    pub static ref EXTENSION_PORT: ReentrantMutex<RefCell<Port>> = {
        let port = chrome.runtime().connect();
        let on_message_cb = create_message_listener(&port);
        port.on_message()
            .add_listener(on_message_cb.into_js_value());
        port.on_disconnect()
            .add_listener(Closure::<dyn Fn(Port)>::new(port_disconnect_handler).into_js_value());
        ReentrantMutex::new(RefCell::new(port))
    };
}

use yewdux::prelude::*;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum StoreStatus {
    #[default]
    Idle,
    Loading,
    Success,
    Failure,
    Error,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct StoreData {
    pub accounts: Mrc<Vec<Rc<Account>>>,
    pub storage_status: StorageStatus,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ContentScriptStore {
    pub page_loading: bool,
    pub alert_input: AlertInput,
    pub verified: bool,
    pub status: StoreStatus,
    pub data: StoreData,
    pub path: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DataAction {
    ResourceFetchStarted(Resource),
    Init(Value),
    ResourceDeleted(Resource, Value),
    ResourceCreated(Resource, Value),
    ResourceDeletionFailed(Resource, Value),
    ResourceCreationFailed(Resource, SessionEventWrapper),
    ResourceDeletionStarted(Resource, Value),
    ResourceCreationStarted(Resource, Value),
    ResourceFetched(Resource, Value, Option<Value>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContentScriptAction {
    PathSet(Option<String>),
}
impl Reducer<ContentScriptStore> for ContentScriptAction {
    fn apply(self, state: Rc<ContentScriptStore>) -> Rc<ContentScriptStore> {
        match self {
            ContentScriptAction::PathSet(path) => ContentScriptStore {
                path,
                ..state.deref().clone()
            }
            .into(),
        }
    }
}
impl Reducer<ContentScriptStore> for DataAction {
    fn apply(self, state: Rc<ContentScriptStore>) -> Rc<ContentScriptStore> {
        match self {
            DataAction::ResourceCreationStarted(_resource, _data) => ContentScriptStore {
                page_loading: true,
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceDeletionStarted(_resource, _data) => {
                ContentScriptStore {
                    page_loading: true,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceFetched(resource, data, _meta) => match resource {
                Resource::Account => {
                    dbg!(&data);
                    let state_data = state.data.clone();
                    let accounts = serde_json::from_value::<Vec<Account>>(data)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|v| Rc::new(v))
                        .collect::<Vec<Rc<Account>>>();
                    dbg!(&accounts);
                    ContentScriptStore {
                        page_loading: false,
                        data: StoreData {
                            storage_status: StorageStatus::Loaded,
                            accounts: Mrc::new(accounts),
                            ..state_data
                        },
                        ..state.deref().clone()
                    }
                    .into()
                }
                _ => {
                    todo!();
                }
            },
            DataAction::ResourceCreated(resource, data) => match resource {
                Resource::Account => {
                    let account = serde_json::from_value::<Account>(data.clone()).unwrap();
                    let mut state_data = state.data.clone();
                    let accounts = state_data.accounts.clone();
                    accounts.borrow_mut().push(Rc::new(account));

                    state_data.accounts = accounts;
                    ContentScriptStore {
                        page_loading: false,
                        data: state_data,
                        ..state.deref().clone()
                    }
                    .into()
                }
                _ => ContentScriptStore {
                    ..state.deref().clone()
                }
                .into(),
            },
            DataAction::ResourceFetchStarted(_resource) => ContentScriptStore {
                page_loading: true,
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceDeleted(resource, data) => {
                let state_data = state.data.clone();
                match resource {
                    Resource::Account => {
                        let account = serde_json::from_value::<Account>(data.clone()).unwrap();
                        state_data
                            .accounts
                            .borrow_mut()
                            .retain(|ac| account.id != ac.id);
                        ContentScriptStore {
                            page_loading: false,
                            data: state_data,
                            ..state.deref().clone()
                        }
                        .into()
                    }
                    _ => ContentScriptStore {
                        ..state.deref().clone()
                    }
                    .into(),
                }
            }
            DataAction::Init(data) => ContentScriptStore {
                verified: data
                    .get("verified")
                    .map_or(false, |v| v.as_bool().unwrap_or(false)),
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceCreationFailed(resource, _session_event_wrapper) => {
                info!("resource creation failed for resource: {:?}", &resource);
                ContentScriptStore {
                    page_loading: false,
                    ..state.deref().clone()
                }
            }
            .into(),
            _ => state,
        }
    }
}

impl Reducer<ContentScriptStore> for LoginAction {
    fn apply(self, store: Rc<ContentScriptStore>) -> Rc<ContentScriptStore> {
        match self {
            LoginAction::LoginStarted => ContentScriptStore {
                page_loading: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginError => ContentScriptStore {
                page_loading: false,
                status: StoreStatus::Error,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginSucceeded => ContentScriptStore {
                page_loading: false,
                verified: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginFailed => ContentScriptStore {
                page_loading: false,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutSucceeded => ContentScriptStore {
                page_loading: false,
                verified: false,
                status: StoreStatus::Success,
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    ..store.deref().clone().data
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutFailed => ContentScriptStore {
                page_loading: false,
                status: StoreStatus::Failure,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutStarted => ContentScriptStore {
                page_loading: true,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Logout => ContentScriptStore {
                verified: false,
                page_loading: false,
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    ..store.deref().clone().data
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Login => {
                dbg!("login event!!!");
                ContentScriptStore {
                    verified: true,
                    page_loading: false,
                    ..store.deref().clone()
                }
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

impl Store for ContentScriptStore {
    fn new() -> Self {
        init_listener(StorageListener);
        ContentScriptStore::load();
        ContentScriptStore::default()
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl ContentScriptStore {
    pub fn load() {
        let acknowledgement = create_request_acknowledgement();
        let init_config = HashMap::new();
        let init_request =
            RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
        EXTENSION_PORT
            .lock()
            .borrow()
            .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = ContentScriptStore;

    fn on_change(&mut self, _state: Rc<Self::Store>) {}
}
