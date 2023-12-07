use browser_rpass::request::SessionEventWrapper;
use browser_rpass::types::StorageStatus;
use gloo::storage::errors::StorageError;
use gloo_utils::document;
use gloo_utils::format::JsValueSerdeExt;
use lazy_static::lazy_static;
use log::debug;
use parking_lot::ReentrantMutex;
use serde_json;
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
    Loading,
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
    pub user_id: Option<String>,
    pub remember_me: bool,
    pub dark_mode: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct PopupStore {
    pub persistent_data: PersistentStoreData,
    pub page_loading: bool,
    pub alert_input: AlertInput,
    pub verified: bool,
    pub data_status: StoreDataStatus,
    pub login_status: LoginStatus,
    pub data: StoreData,
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoginAction {
    LoginStarted(String, Value),
    LoginError(Value, String),
    LoginSucceeded(Value),
    LoginFailed(Value),
    LogoutSucceeded(Value),
    LogoutFailed(Value),
    LogoutStarted(Value),
    Logout(Value),
    Login(String, Value),
    LoginIdle,
    RememberMe(bool),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DataAction {
    ResourceFetchStarted(Resource),
    Init(Value),
    ResourceDeleted(Resource, Value),
    ResourceCreated(Resource, Value),
    ResourceEdited(Resource, Value, String),
    ResourceDeletionFailed(Resource, Value),
    ResourceEditionFailed(Resource, SessionEventWrapper),
    ResourceCreationFailed(Resource, SessionEventWrapper),
    ResourceDeletionStarted(Resource, Value),
    ResourceEditionStarted(Resource, Value),
    ResourceCreationStarted(Resource, Value),
    ResourceFetched(Resource, Value, Option<Value>),
    Idle,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PopupAction {
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
            DataAction::ResourceFetched(resource, data, _meta) => match resource {
                Resource::Account => {
                    let state_data = state.data.clone();
                    let accounts = serde_json::from_value::<Vec<Account>>(data)
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
                        data_status: StoreDataStatus::FetchSuccess,
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
            DataAction::ResourceEdited(resource, data, id) => match resource {
                Resource::Account => {
                    let account = serde_json::from_value::<Account>(data.clone()).unwrap();
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
            DataAction::ResourceDeleted(resource, data) => {
                let state_data = state.data.clone();
                match resource {
                    Resource::Account => {
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
            DataAction::Init(data) => PopupStore {
                verified: data
                    .get("verified")
                    .map_or(false, |v| v.as_bool().unwrap_or(false)),
                ..state.deref().clone()
            }
            .into(),
            DataAction::ResourceCreationFailed(resource, _session_event_wrapper) => {
                PopupStore {
                    page_loading: false,
                    data_status: StoreDataStatus::CreationFailed,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceEditionFailed(resource, _session_event_wrapper) => {
                PopupStore {
                    page_loading: false,
                    data_status: StoreDataStatus::EditionFailed,
                    ..state.deref().clone()
                }
            }
            .into(),
            DataAction::ResourceDeletionFailed(resource, _session_event_wrapper) => PopupStore {
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
        }
    }
}

impl Reducer<PopupStore> for LoginAction {
    fn apply(self, store: Rc<PopupStore>) -> Rc<PopupStore> {
        match self {
            LoginAction::LoginStarted(user_id, data) => PopupStore {
                page_loading: true,
                persistent_data: PersistentStoreData {
                    user_id: Some(user_id),
                    remember_me: store.persistent_data.remember_me,
                    dark_mode: store.persistent_data.dark_mode,
                },
                login_status: LoginStatus::Loading,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginIdle => PopupStore {
                page_loading: false,
                login_status: LoginStatus::Idle,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginError(data, user_id) => {
                debug!("LoginError: {:?}", user_id);
                PopupStore {
                    page_loading: false,
                    login_status: LoginStatus::LoginError,
                    ..store.deref().clone()
                }
            }
            .into(),
            LoginAction::LoginSucceeded(data) => PopupStore {
                page_loading: false,
                verified: true,
                login_status: LoginStatus::LoginSuccess,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LoginFailed(data) => PopupStore {
                page_loading: false,
                login_status: LoginStatus::LoginFailed,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutSucceeded(data) => PopupStore {
                page_loading: false,
                verified: false,
                persistent_data: PersistentStoreData {
                    user_id: if store.persistent_data.remember_me {
                        store.persistent_data.user_id.clone()
                    } else {
                        None
                    },
                    remember_me: store.persistent_data.remember_me,
                    dark_mode: store.persistent_data.dark_mode,
                },
                login_status: LoginStatus::LogoutSuccess,
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    ..store.deref().clone().data
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutFailed(data) => PopupStore {
                page_loading: false,
                login_status: LoginStatus::LogoutFailed,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::LogoutStarted(data) => PopupStore {
                page_loading: true,
                login_status: LoginStatus::Loading,
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Logout(data) => PopupStore {
                verified: false,
                page_loading: false,
                persistent_data: PersistentStoreData {
                    remember_me: store.persistent_data.remember_me,
                    user_id: if store.persistent_data.remember_me {
                        store.persistent_data.user_id.clone()
                    } else {
                        None
                    },
                    dark_mode: store.persistent_data.dark_mode,
                },
                login_status: LoginStatus::LoggedOut,
                data: StoreData {
                    accounts: Mrc::new(vec![]),
                    ..store.deref().clone().data
                },
                ..store.deref().clone()
            }
            .into(),
            LoginAction::Login(user_id, data) => {
                PopupStore {
                    verified: true,
                    page_loading: false,
                    persistent_data: PersistentStoreData {
                        remember_me: store.persistent_data.remember_me,
                        user_id: Some(user_id),
                        dark_mode: store.persistent_data.dark_mode,
                    },
                    login_status: LoginStatus::LoggedIn,
                    ..store.deref().clone()
                }
            }
            .into(),
            LoginAction::RememberMe(remember_me) => {
                PopupStore {
                    persistent_data: PersistentStoreData {
                        remember_me,
                        user_id: if remember_me {
                            store.persistent_data.user_id.clone()
                        } else {
                            None
                        },
                        dark_mode: store.persistent_data.dark_mode,
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
        log::debug!("PopupStore::new");
        init_listener(StorageListener);
        // PopupStore::init();
        PopupStore::default()
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl PopupStore {
    pub fn save<T: Serialize>(state: &T, area: store::StorageArea) -> Result<(), StorageError> {
        let value = serde_json::to_string(state)
            .map_err(|serde_error| StorageError::SerdeError(serde_error))?;

        wasm_bindgen_futures::spawn_local(async move {
            match area {
                store::StorageArea::Local => {
                    let _ = chrome
                        .storage()
                        .local()
                        .set_string_item(type_name::<T>().to_owned(), value, area)
                        .await;
                }
                store::StorageArea::Sync => {
                    let _ = chrome
                        .storage()
                        .sync()
                        .set_string_item(type_name::<T>().to_owned(), value, area)
                        .await;
                }
                store::StorageArea::Session => {
                    let _ = chrome
                        .storage()
                        .session()
                        .set_string_item(type_name::<T>().to_owned(), value, area)
                        .await;
                }
            }
        });
        Ok(())
    }

    pub fn init() {
        let acknowledgement = create_request_acknowledgement();
        let init_config = HashMap::new();
        let init_request =
            RequestEnum::create_init_request(init_config, Some(acknowledgement.clone()), None);
        EXTENSION_PORT
            .lock()
            .borrow()
            .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
    }
    pub async fn load() -> Option<PersistentStoreData> {
        let loaded = chrome
            .storage()
            .local()
            .get_item(
                &type_name::<PersistentStoreData>(),
                store::StorageArea::Local,
            )
            .await;
        if let Ok(value) = loaded {
            let parsed = <JsValue as JsValueSerdeExt>::into_serde::<String>(&value);
            if let Ok(json_string) = parsed {
                let state = serde_json::from_str::<PersistentStoreData>(&json_string);
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
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = PopupStore;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        if let Err(err) =
            Self::Store::save(&(state.as_ref().persistent_data), store::StorageArea::Local)
        {
            println!("Error saving state to storage: {:?}", err);
        } else {
        }
    }
}
