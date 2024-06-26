use browser_rpass::dbg;
use browser_rpass::js_binding::extension_api::*;
use browser_rpass::request::DataFieldType;
use browser_rpass::request::SessionEvent;
use browser_rpass::types::Account;
use browser_rpass::types::Resource;
use browser_rpass::types::StorageStatus;
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
#[allow(unused_imports)]
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
    let mut init_config = HashMap::new();
    init_config.insert(DataFieldType::ContentScript, "true".to_string());
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
    pub store_id: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct ContentScriptStore {
    pub page_loading: bool,
    pub alert_input: AlertInput,
    pub status: StoreStatus,
    pub data: StoreData,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoginAction {
    LoginStarted,
    LoginError,
    LoginSucceeded(Option<HashMap<DataFieldType, Value>>),
    LoginFailed,
    LogoutSucceeded,
    LogoutFailed,
    LogoutStarted,
    Logout,
    Login(Option<HashMap<DataFieldType, Value>>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DataAction {
    ResourceFetchStarted(Resource),
    Init(HashMap<DataFieldType, Value>),
    ResourceDeleted(Resource, HashMap<DataFieldType, Value>),
    ResourceCreated(Resource, HashMap<DataFieldType, Value>),
    ResourceDeletionFailed(Resource, HashMap<DataFieldType, Value>),
    ResourceCreationFailed(Resource, SessionEvent),
    ResourceDeletionStarted(Resource, HashMap<DataFieldType, Value>),
    ResourceCreationStarted(Resource, HashMap<DataFieldType, Value>),
    ResourceFetched(Resource, HashMap<DataFieldType, Value>, Option<Value>),
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
            DataAction::ResourceFetched(resource, mut data, _meta) => match resource {
                Resource::Account => {
                    dbg!(&data);
                    let state_data = state.data.clone();
                    let accounts = data.remove(&DataFieldType::Data).unwrap_or_default();
                    let accounts = serde_json::from_value::<Vec<Account>>(accounts)
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
            DataAction::ResourceCreated(resource, mut data) => match resource {
                Resource::Account => {
                    let account = data.remove(&DataFieldType::Data).unwrap_or_default();
                    let account = serde_json::from_value::<Account>(account).unwrap();
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
            DataAction::ResourceDeleted(resource, mut data) => {
                let state_data = state.data.clone();
                match resource {
                    Resource::Account => {
                        let account = data.remove(&DataFieldType::Data).unwrap_or_default();
                        let account = serde_json::from_value::<Account>(account).unwrap();
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
            DataAction::Init(data) => {
                let default_store_available = data
                    .get(&DataFieldType::DefaultStoreAvailable)
                    .map_or(false, |v| v.as_bool().unwrap_or(false));
                let store_id = data
                    .get(&DataFieldType::DefaultStoreID)
                    .map_or(None, |v| v.as_str().map_or(None, |v| Some(v.to_string())));
                ContentScriptStore {
                    data: StoreData {
                        store_id,
                        verified: default_store_available,
                        ..state.data.clone()
                    },
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceCreationFailed(_resource, _session_event) => {
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
            LoginAction::LoginSucceeded(data) => ContentScriptStore {
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    verified: true,
                    store_id: data
                        .as_ref()
                        .and_then(|v| v.get(&DataFieldType::DefaultStoreID))
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    ..store.deref().clone().data
                },
                page_loading: false,
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
                status: StoreStatus::Success,
                data: StoreData {
                    verified: false,
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
                page_loading: false,
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    verified: false,

                    ..store.deref().clone().data
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Login(data) => {
                ContentScriptStore {
                    data: StoreData {
                        accounts: Mrc::new(vec![]),
                        verified: true,
                        store_id: data
                            .as_ref()
                            .and_then(|v| v.get(&DataFieldType::DefaultStoreID))
                            .and_then(|v| v.as_str())
                            .map(|v| v.to_string()),
                        ..store.deref().clone().data
                    },
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
        let mut init_config = HashMap::new();
        init_config.insert(DataFieldType::ContentScript, "true".to_string());
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
