use browser_rpass::js_binding::extension_api::*;
use browser_rpass::request::DataFieldType;
use browser_rpass::request::SessionEventWrapper;
use browser_rpass::types::*;
use gloo::storage::errors::StorageError;
use gloo_utils::document;
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use log::debug;
use parking_lot::ReentrantMutex;
use serde_json;
use serde_json::json;
use wasm_bindgen::prelude::Closure;
use yewdux::mrc::Mrc;

use browser_rpass::store;

use crate::event_handlers::extension_message_listener::create_message_listener;
use crate::Resource;
use browser_rpass::response::RequestEnum;
pub use browser_rpass::util::*;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::any::type_name;
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

use crate::Account;

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum StoreDataStatus {
    #[default]
    Idle,
    Loading,
    CreationStarted,
    StoreCreationStarted(Option<RequestEnum>, String),
    StoreCreationFailed(HashMap<DataFieldType, Value>, String),
    StoreCreated(HashMap<DataFieldType, Value>, String),
    StoreDeletionStarted(Option<RequestEnum>, String),
    StoreDeletionFailed(HashMap<DataFieldType, Value>, String),
    StoreDeleted(HashMap<DataFieldType, Value>, String),
    CreationSuccess,
    CreationFailed,
    DeletionStarted,
    DeletionSuccess,
    DeletionFailed,
    EditionStarted,
    EditionSuccess,
    EditionFailed,
    FetchStarted,
    FetchSuccess,
    FetchFailed,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub enum LoginStatus {
    LoginFailed,
    LoggedIn,
    LoggedOut,
    #[default]
    Idle,
    LoginStarted(String),
    LogoutStarted,
    LoginSuccess,
    LoginError,
    LogoutSuccess,
    LogoutError,
    LogoutFailed,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct StoreData {
    pub accounts: Mrc<Vec<Rc<Account>>>,
    pub storage_status: StorageStatus,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct PersistentStoreData {
    pub store_id: Option<String>,
    pub store_activated: bool,
    pub remember_me: bool,
    pub dark_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct PopupStore {
    pub persistent_data: PersistentStoreData,
    pub page_loading: bool,
    pub alert_input: AlertInput,
    pub data_status: StoreDataStatus,
    pub login_status: LoginStatus,
    pub data: StoreData,
    pub store_ids: Vec<String>,
    pub keys: Vec<Key>,
    pub path: Option<String>,
    pub window_id: Option<String>,
    pub default_store_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoginAction {
    LoginStarted(String, HashMap<DataFieldType, Value>),
    LoginError(HashMap<DataFieldType, Value>, String),
    LoginSucceeded(HashMap<DataFieldType, Value>),
    LoginFailed(HashMap<DataFieldType, Value>),
    LogoutSucceeded(HashMap<DataFieldType, Value>),
    LogoutFailed(HashMap<DataFieldType, Value>),
    LogoutStarted(HashMap<DataFieldType, Value>),
    Logout(String, HashMap<DataFieldType, Value>),
    Login(String, HashMap<DataFieldType, Value>),
    LoginIdle,
    RememberMe(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DataAction {
    StoreCreated(HashMap<DataFieldType, Value>, String),
    StoreCreationFailed(HashMap<DataFieldType, Value>, String),
    StoreCreationStarted(Option<RequestEnum>, String),
    StoreDeleted(HashMap<DataFieldType, Value>, String),
    StoreDeletionFailed(HashMap<DataFieldType, Value>, String),
    StoreDeletionStarted(Option<RequestEnum>, String),
    StoreIdSet(String),
    ResourceFetchStarted(Resource),
    Init(HashMap<DataFieldType, Value>),
    ResourceDeleted(Resource, HashMap<DataFieldType, Value>),
    ResourceCreated(Resource, HashMap<DataFieldType, Value>),
    ResourceEdited(Resource, HashMap<DataFieldType, Value>, String),
    ResourceDeletionFailed(Resource, HashMap<DataFieldType, Value>),
    ResourceEditionFailed(Resource, SessionEventWrapper),
    ResourceCreationFailed(Resource, SessionEventWrapper),
    ResourceDeletionStarted(Resource, HashMap<DataFieldType, Value>),
    ResourceEditionStarted(Resource, HashMap<DataFieldType, Value>),
    ResourceCreationStarted(Resource, HashMap<DataFieldType, Value>),
    ResourceFetched(Resource, HashMap<DataFieldType, Value>, Option<Value>),
    Idle,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PopupAction {
    WindowIdSet(String),
    PathSet(Option<String>),
    DarkModeToggle,
}
impl Reducer<PopupStore> for PopupAction {
    fn apply(self, state: Rc<PopupStore>) -> Rc<PopupStore> {
        match self {
            PopupAction::PathSet(path) => PopupStore {
                path,
                ..state.deref().clone()
            }
            .into(),
            PopupAction::WindowIdSet(window_id) => PopupStore {
                window_id: Some(window_id),
                ..state.deref().clone()
            }
            .into(),
            PopupAction::DarkModeToggle => {
                let dark_mode = !state.persistent_data.dark_mode;
                if dark_mode {
                    let _ = document().body().unwrap().set_class_name("dark");
                } else {
                    let _ = document().body().unwrap().class_list().remove_1("dark");
                }
                PopupStore {
                    persistent_data: PersistentStoreData {
                        dark_mode,
                        ..state.persistent_data.clone()
                    },
                    ..state.deref().clone()
                }
                .into()
            }
        }
    }
}
impl Reducer<PopupStore> for DataAction {
    fn apply(self, state: Rc<PopupStore>) -> Rc<PopupStore> {
        match self {
            DataAction::StoreIdSet(store_id) => {
                PopupStore {
                    persistent_data: PersistentStoreData {
                        store_id: Some(store_id),
                        ..state.persistent_data.clone()
                    },
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceCreationStarted(_resource, _data) => PopupStore {
                page_loading: true,
                data_status: StoreDataStatus::CreationStarted,
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceEditionStarted(_resource, _data) => PopupStore {
                page_loading: true,
                data_status: StoreDataStatus::EditionStarted,
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceDeletionStarted(_resource, _data) => {
                PopupStore {
                    page_loading: true,
                    data_status: StoreDataStatus::DeletionStarted,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceFetched(resource, mut data, _meta) => match resource {
                Resource::Account => {
                    let state_data = state.data.clone();
                    let accounts = data.remove(&DataFieldType::Data).unwrap_or_default();
                    let accounts = serde_json::from_value::<Vec<Account>>(accounts)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|v| Rc::new(v))
                        .collect::<Vec<Rc<Account>>>();
                    PopupStore {
                        page_loading: false,
                        data: StoreData {
                            storage_status: StorageStatus::Loaded,
                            accounts: Mrc::new(accounts),
                            ..state_data
                        },
                        persistent_data: PersistentStoreData {
                            store_id: data
                                .get(&DataFieldType::StoreID)
                                .map_or(state.persistent_data.store_id.clone(), |v| {
                                    Some(v.as_str().unwrap().to_string())
                                }),
                            store_activated: true,
                            ..state.persistent_data.clone()
                        },
                        data_status: StoreDataStatus::FetchSuccess,
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
                    let state_data = state.data.clone();
                    let mut accounts = state_data.accounts.borrow_mut();
                    accounts.push(Rc::new(account));
                    drop(accounts);
                    PopupStore {
                        page_loading: false,
                        data: state_data,
                        data_status: StoreDataStatus::CreationSuccess,
                        ..state.deref().clone()
                    }
                    .into()
                }
                _ => PopupStore {
                    ..state.deref().clone()
                }
                .into(),
            },
            DataAction::ResourceEdited(resource, mut data, id) => match resource {
                Resource::Account => {
                    let account = data.remove(&DataFieldType::Data).unwrap_or_default();
                    let account = serde_json::from_value::<Account>(account).unwrap();
                    let state_data = state.data.clone();
                    let mut accounts = state_data.accounts.borrow_mut();
                    let idx = accounts.iter().position(|ac| ac.id == id).unwrap();
                    accounts[idx] = Rc::new(account);
                    drop(accounts);
                    PopupStore {
                        page_loading: false,
                        data: state_data,
                        data_status: StoreDataStatus::EditionSuccess,
                        ..state.deref().clone()
                    }
                    .into()
                }
                _ => PopupStore {
                    ..state.deref().clone()
                }
                .into(),
            },
            DataAction::ResourceFetchStarted(_resource) => PopupStore {
                page_loading: true,
                data_status: StoreDataStatus::FetchStarted,
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceDeleted(resource, mut data) => {
                let state_data = state.data.clone();
                match resource {
                    Resource::Account => {
                        let data = data.remove(&DataFieldType::Data).unwrap_or_default();
                        let map = data.as_object().unwrap();
                        let deleted_id = map.get("id").unwrap().as_str().unwrap().to_owned();
                        state_data
                            .accounts
                            .borrow_mut()
                            .retain(|ac| deleted_id != ac.id);
                        PopupStore {
                            page_loading: false,
                            data: state_data,
                            data_status: StoreDataStatus::DeletionSuccess,
                            ..state.deref().clone()
                        }
                        .into()
                    }
                    _ => PopupStore {
                        ..state.deref().clone()
                    }
                    .into(),
                }
            }
            DataAction::Init(data) => {
                let keys = data.get(&DataFieldType::Keys).map_or(vec![], |v| {
                    v.as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|v| serde_json::from_value::<Key>(v.clone()).unwrap())
                        .collect::<Vec<Key>>()
                });
                PopupStore {
                    persistent_data: PersistentStoreData {
                        ..state.persistent_data.clone()
                    },
                    page_loading: false,
                    store_ids: data
                        .get(&DataFieldType::StoreIDList)
                        .map_or(vec![], |v| {
                            v.as_array()
                                .unwrap_or(&vec![])
                                .iter()
                                .map(|v| v.as_str().unwrap_or("").to_owned())
                                .collect::<Vec<String>>()
                        })
                        .into_iter()
                        .filter(|v| !v.is_empty())
                        .collect::<Vec<String>>(),
                    keys,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceCreationFailed(_resource, _session_event_wrapper) => {
                PopupStore {
                    page_loading: false,
                    data_status: StoreDataStatus::CreationFailed,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceEditionFailed(_resource, _session_event_wrapper) => {
                PopupStore {
                    page_loading: false,
                    data_status: StoreDataStatus::EditionFailed,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceDeletionFailed(_resource, _session_event_wrapper) => PopupStore {
                page_loading: false,
                data_status: StoreDataStatus::DeletionFailed,
                ..state.deref().clone()
            }
            .into(),
            DataAction::Idle => PopupStore {
                page_loading: false,
                data_status: StoreDataStatus::Idle,
                ..state.deref().clone()
            }
            .into(),
            DataAction::StoreCreationStarted(request, acknowledgement) => PopupStore {
                page_loading: true,
                data_status: StoreDataStatus::StoreCreationStarted(request, acknowledgement),
                ..state.deref().clone()
            }
            .into(),
            DataAction::StoreCreated(data, store_id) => {
                let mut store_ids = state.store_ids.clone();
                store_ids.push(store_id.clone());
                PopupStore {
                    page_loading: false,
                    store_ids,
                    data_status: StoreDataStatus::StoreCreated(data, store_id),
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::StoreCreationFailed(data, acknowledgement) => PopupStore {
                page_loading: false,
                data_status: StoreDataStatus::StoreCreationFailed(data, acknowledgement),
                ..state.deref().clone()
            }
            .into(),
            DataAction::StoreDeletionStarted(request, acknowledgement) => PopupStore {
                page_loading: true,
                data_status: StoreDataStatus::StoreDeletionStarted(request, acknowledgement),
                ..state.deref().clone()
            }
            .into(),
            DataAction::StoreDeleted(data, store_id) => {
                let mut store_ids = state.store_ids.clone();
                store_ids.retain(|v| v != &store_id);
                let current_store_id = state.persistent_data.store_id.clone();
                let (persistent_data, store_data) =
                    if current_store_id.is_some_and(|id| id == store_id) {
                        (
                            PersistentStoreData {
                                store_id: None,
                                store_activated: false,
                                ..state.persistent_data.clone()
                            },
                            StoreData {
                                accounts: Mrc::new(vec![]),
                                ..state.deref().clone().data
                            },
                        )
                    } else {
                        (state.persistent_data.clone(), state.data.clone())
                    };
                PopupStore {
                    page_loading: false,
                    store_ids,
                    persistent_data,
                    data: store_data,
                    data_status: StoreDataStatus::StoreDeleted(data, store_id.clone()),
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::StoreDeletionFailed(data, acknowledgement) => PopupStore {
                page_loading: false,
                data_status: StoreDataStatus::StoreDeletionFailed(data, acknowledgement),
                ..state.deref().clone()
            }
            .into(),
        }
    }
}

impl Reducer<PopupStore> for LoginAction {
    fn apply(self, store: Rc<PopupStore>) -> Rc<PopupStore> {
        match self {
            LoginAction::LoginStarted(store_id, _data) => PopupStore {
                page_loading: true,
                persistent_data: PersistentStoreData {
                    ..store.persistent_data.clone()
                },
                login_status: LoginStatus::LoginStarted(store_id.clone()),
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginIdle => PopupStore {
                login_status: LoginStatus::Idle,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginError(_data, _store_id) => {
                PopupStore {
                    page_loading: false,
                    login_status: LoginStatus::LoginError,
                    ..store.deref().clone()
                }
            }
            .into(),
            LoginAction::LoginSucceeded(data) => PopupStore {
                page_loading: false,
                persistent_data: PersistentStoreData {
                    store_id: data
                        .get(&DataFieldType::StoreID)
                        .map_or(store.persistent_data.store_id.clone(), |v| {
                            Some(v.as_str().unwrap().to_string())
                        }),
                    store_activated: true,
                    ..store.persistent_data.clone()
                },
                login_status: LoginStatus::LoginSuccess,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginFailed(_data) => PopupStore {
                page_loading: false,
                login_status: LoginStatus::LoginFailed,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutSucceeded(_data) => {
                debug!("Logout succeeded");
                PopupStore {
                    page_loading: false,
                    persistent_data: PersistentStoreData {
                        store_id: if store.persistent_data.remember_me {
                            store.persistent_data.store_id.clone()
                        } else {
                            None
                        },
                        store_activated: false,
                        ..store.persistent_data
                    },
                    login_status: LoginStatus::LogoutSuccess,
                    data: StoreData {
                        accounts: Mrc::new(vec![]),
                        ..store.deref().clone().data
                    },
                    ..store.deref().clone()
                }
            }
            .into(),
            LoginAction::LogoutFailed(_data) => PopupStore {
                page_loading: false,
                login_status: LoginStatus::LogoutFailed,
                persistent_data: PersistentStoreData {
                    ..store.persistent_data.clone()
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutStarted(_data) => PopupStore {
                page_loading: true,
                login_status: LoginStatus::LogoutStarted,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Logout(store_id, _data) => {
                debug!("Logout");
                let current_store_id = store.persistent_data.store_id.as_ref();
                if current_store_id.is_some_and(|v| *v != store_id) {
                    store
                } else {
                    PopupStore {
                        page_loading: false,
                        persistent_data: PersistentStoreData {
                            store_id: if store.persistent_data.remember_me {
                                store.persistent_data.store_id.clone()
                            } else {
                                None
                            },
                            store_activated: false,
                            ..store.persistent_data
                        },
                        login_status: LoginStatus::LoggedOut,
                        data: StoreData {
                            accounts: Mrc::new(vec![]),
                            ..store.deref().clone().data
                        },
                        ..store.deref().clone()
                    }
                    .into()
                }
            }
            LoginAction::Login(store_id, data) => {
                PopupStore {
                    page_loading: false,
                    persistent_data: PersistentStoreData {
                        store_activated: {
                            if let LoginStatus::LoginStarted(current_store_id) =
                                store.login_status.clone()
                            {
                                &store_id == &current_store_id
                            } else {
                                store.persistent_data.store_activated
                            }
                        },
                        ..store.persistent_data.clone()
                    },
                    default_store_id: {
                        if data
                            .get(&DataFieldType::IsDefault)
                            .map_or(false, |v| v.as_bool().unwrap_or(false))
                        {
                            Some(store_id)
                        } else {
                            store.default_store_id.clone()
                        }
                    },
                    login_status: LoginStatus::LoggedIn,
                    ..store.deref().clone()
                }
            }
            .into(),
            LoginAction::RememberMe(remember_me) => {
                PopupStore {
                    persistent_data: PersistentStoreData {
                        store_id: if remember_me {
                            store.persistent_data.store_id.clone()
                        } else {
                            None
                        },
                        remember_me,
                        ..store.persistent_data
                    },
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

impl Store for PopupStore {
    fn new() -> Self {
        init_listener(StorageListener);
        PopupStore::default()
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl PopupStore {
    pub fn save(store: Rc<Self>, area: store::StorageArea) -> Result<(), StorageError> {
        let window_id = store.window_id.clone().unwrap();
        let json = json!({type_name::<PersistentStoreData>(): store.persistent_data});
        let value = serde_json::to_string(&json)
            .map_err(|serde_error| StorageError::SerdeError(serde_error))?;
        wasm_bindgen_futures::spawn_local({
            let window_id = window_id.clone();
            let value = value.clone();
            async move {
                let _old_data = chrome
                    .storage()
                    .local()
                    .get_item(&window_id, store::StorageArea::Local)
                    .await;
                let _ = chrome
                    .storage()
                    .local()
                    .set_string_item("default".to_string(), value, area)
                    .await;
            }
        });
        wasm_bindgen_futures::spawn_local(async move {
            match area {
                store::StorageArea::Local => {
                    let _ = chrome
                        .storage()
                        .local()
                        .set_string_item(window_id, value, area)
                        .await;
                }
                store::StorageArea::Sync => {
                    let _ = chrome
                        .storage()
                        .sync()
                        .set_string_item(window_id, value, area)
                        .await;
                }
                store::StorageArea::Session => {
                    let _ = chrome
                        .storage()
                        .session()
                        .set_string_item(window_id, value, area)
                        .await;
                }
            }
        });
        Ok(())
    }

    pub async fn load_window_storage(window_id: &str) -> Option<PersistentStoreData> {
        let window_storage = chrome
            .storage()
            .local()
            .get_item(window_id, store::StorageArea::Local)
            .await;
        if let Ok(window_storage) = window_storage {
            if let Ok(window_storage) =
                <JsValue as JsValueSerdeExt>::into_serde::<String>(&window_storage)
            {
                let window_storage =
                    serde_json::from_str::<serde_json::Value>(&window_storage).unwrap();
                if let Some(persistent_data) =
                    window_storage.get(&type_name::<PersistentStoreData>())
                {
                    let state =
                        serde_json::from_value::<PersistentStoreData>(persistent_data.clone());
                    if let Ok(state) = state {
                        return Some(state);
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                let default_storage = chrome
                    .storage()
                    .local()
                    .get_item("default", store::StorageArea::Local)
                    .await;
                if let Ok(default_storage) = default_storage {
                    if let Ok(default_storage) =
                        <JsValue as JsValueSerdeExt>::into_serde::<String>(&default_storage)
                    {
                        let default_storage =
                            serde_json::from_str::<serde_json::Value>(&default_storage).unwrap();
                        if let Some(persistent_data) =
                            default_storage.get(&type_name::<PersistentStoreData>())
                        {
                            let state = serde_json::from_value::<PersistentStoreData>(
                                persistent_data.clone(),
                            );
                            if let Ok(state) = state {
                                return Some(state);
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = PopupStore;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        if let Err(err) = Self::Store::save(state.clone(), store::StorageArea::Local) {
            println!("Error saving state to storage: {:?}", err);
        } else {
        }
    }
}
