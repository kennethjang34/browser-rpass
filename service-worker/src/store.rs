use crate::event_handlers::native_message_handler::process_native_message;
pub use crate::Resource;
use crate::{api, StorageStatus};
use browser_rpass::dbg;
use browser_rpass::request::{RequestEnumTrait, SessionEventType};
use browser_rpass::response::{CreateResponse, EditResponse, FetchResponse};
use browser_rpass::types::Account;
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use parking_lot::ReentrantMutex;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashSet;
use std::{any::type_name, collections::HashMap, ops::Deref, rc::Rc, sync::Mutex};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use yewdux::mrc::Mrc;

use yewdux::prelude::Reducer;

pub use browser_rpass::{
    request::{RequestEnum, SessionEvent, SessionEventWrapper},
    store::AsyncCallback,
    store::MESSAGE_ACKNOWLEDGEMENTS_NATIVE,
    store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP,
    util::{chrome, Port},
};
use gloo::storage::errors::StorageError;
use lazy_static::lazy_static;
pub enum StorageArea {
    Session,
}
use yewdux::{
    prelude::{init_listener, Dispatch, Listener},
    store::Store,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SessionAction {
    Login,
    LoginError,
    Logout,
    LogoutError,
    DataFetched(FetchResponse),
    DataLoading(Option<String>),
    DataCreated(CreateResponse),
    DataEdited(EditResponse),
    DataDeleted(Resource, Value),
    DataDeletionFailed(Resource, String),
    DataCreationFailed(Resource, Value, Option<RequestEnum>),
    DataEditFailed(Resource, Value, Option<RequestEnum>),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionActionWrapper {
    pub meta: Option<Value>,
    pub action: SessionAction,
}

fn native_port_disconnect_handler(_port: Port) {
    let new_port = chrome.runtime().connect_native("com.rpass");
    new_port
        .on_disconnect()
        .add_listener(Closure::<dyn Fn(Port)>::new(native_port_disconnect_handler).into_js_value());
    new_port
        .on_message()
        .add_listener(Closure::<dyn Fn(String)>::new(native_port_message_handler).into_js_value());
    #[allow(unused_mut)]
    let mut init_config = HashMap::new();
    let init_request = RequestEnum::create_init_request(init_config, None, None);
    new_port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
    if let Ok(mut borrowed) = NATIVE_PORT.lock().try_borrow_mut() {
        *borrowed = new_port;
    }
}
fn native_port_message_handler(msg: String) {
    match serde_json::from_slice::<serde_json::Value>(&msg.as_bytes()) {
        Ok(parsed_response) => {
            let acknowledgement = parsed_response.get("acknowledgement").cloned().unwrap();
            let acknowledgement = acknowledgement.as_str().unwrap();
            dbg!(&parsed_response);
            let _ = process_native_message(
                parsed_response,
                NATIVE_PORT.lock().borrow().clone(),
                REQUEST_MAP.lock().unwrap().get(acknowledgement),
                Some(json!({"acknowledgement":acknowledgement})),
            );
        }
        Err(e) => {
            error!(
                "error happend while parsing:{:?}. Error message: {:?}",
                msg, e
            );
        }
    }
}
lazy_static! {
    pub static ref NATIVE_PORT: ReentrantMutex<RefCell<Port>> = {
        let port = chrome.runtime().connect_native("com.rpass");
        port.on_disconnect().add_listener(
            Closure::<dyn Fn(Port)>::new(native_port_disconnect_handler).into_js_value(),
        );
        port.on_message().add_listener(
            Closure::<dyn Fn(String)>::new(native_port_message_handler).into_js_value(),
        );
        let mut init_config = HashMap::new();
        let init_request = RequestEnum::create_init_request(init_config, None, None);
        port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
        ReentrantMutex::new(RefCell::new(port))
    };
}
lazy_static! {
    pub static ref EXTENSION_PORT: Mutex<HashMap<String, Port>> = Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref LISTENER_PORT: Mutex<HashMap<Resource, HashSet<String>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref REQUEST_MAP: Mutex<HashMap<String, RequestEnum>> = Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref PORT_ID_MAP: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct StoreData {
    pub accounts: Mrc<Vec<Rc<Account>>>,
    pub storage_status: StorageStatus,
}
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SessionStore {
    pub user: (Option<String>, Option<String>),
    pub data: StoreData,
}

impl Store for SessionStore {
    fn new() -> Self {
        init_listener(StorageListener);
        let loaded = SessionStore::load().ok().flatten().unwrap_or_default();
        loaded
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl SessionStore {
    pub fn load() -> Result<Option<SessionStore>, StorageError> {
        let storage = chrome.storage().session();
        let value = storage.get_string_value_sync(type_name::<SessionStore>());
        match value {
            Ok(value) => {
                if let Some(value) = value {
                    let state = serde_json::from_str(&value);
                    if let Ok(state) = state {
                        Ok(Some(state))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(err) => Err(err),
        }
    }
    pub fn save<T: Serialize>(state: &T, _area: StorageArea) -> Result<(), StorageError> {
        let value = serde_json::to_string(state)
            .map_err(|serde_error| StorageError::SerdeError(serde_error))?;
        chrome
            .storage()
            .session()
            .set_string_item_sync(type_name::<T>().to_owned(), value);
        Ok(())
    }
}

impl Reducer<SessionStore> for SessionActionWrapper {
    fn apply(self, store: Rc<SessionStore>) -> Rc<SessionStore> {
        let meta = self.meta;
        let mut extension_port_name: Option<String> = None;
        let session_action = self.action;
        let mut clear_ports = false;
        let (session_store, session_event) = match session_action {
            SessionAction::Login => {
                let passphrase = {
                    if let Some(ref meta) = meta {
                        if let Some(passphrase) = meta.get("passphrase") {
                            Some(passphrase.as_str().unwrap().to_owned())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                let user_id = {
                    if let Some(ref meta) = meta {
                        if let Some(user_id) = meta.get("user_id") {
                            Some(user_id.as_str().unwrap().to_owned())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                (
                    SessionStore {
                        user: (user_id.clone(), passphrase),
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        event_type: SessionEventType::Login,
                        data: Some(json!({"verified":true,"user_id":user_id.clone()})),
                        meta,
                        resource: Some(vec![Resource::Auth]),
                        is_global: true,
                    }),
                )
            }
            SessionAction::LoginError => (
                SessionStore {
                    user: (store.user.0.clone(), None),
                    ..store.deref().clone()
                }
                .into(),
                Some(SessionEvent {
                    event_type: SessionEventType::LoginError,
                    data: None,
                    meta,
                    resource: Some(vec![Resource::Auth]),
                    is_global: true,
                }),
            ),
            SessionAction::Logout => {
                if (*store).user.1.is_some() {
                    clear_ports = true;
                    (
                        SessionStore {
                            user: (store.user.0.clone(), None),
                            data: StoreData::default(),
                            ..SessionStore::default().clone()
                        }
                        .into(),
                        Some(SessionEvent {
                            event_type: SessionEventType::Logout,
                            data: Some(json!({"verified":false})),
                            meta,
                            resource: Some(vec![Resource::Auth]),
                            is_global: true,
                        }),
                    )
                } else {
                    (store, None)
                }
            }
            SessionAction::DataDeleted(resource, data) => match resource.clone() {
                Resource::Account => {
                    let account_deleted = serde_json::from_value::<Account>(data.clone()).unwrap();
                    let mut account_vec = store.data.accounts.borrow_mut();
                    let index = account_vec
                        .iter()
                        .position(|ac| account_deleted.id == ac.id);
                    if let Some(index) = index {
                        account_vec.remove(index);
                    } else {
                        dbg!("account not found");
                    }
                    (
                        SessionStore {
                            data: StoreData {
                                accounts: store.data.accounts.clone(),
                                storage_status: store.data.storage_status.clone(),
                            },
                            ..store.deref().clone()
                        }
                        .into(),
                        Some(SessionEvent {
                            event_type: SessionEventType::Delete,
                            data: Some(data),
                            meta,
                            resource: Some(vec![resource]),
                            is_global: true,
                        }),
                    )
                }
                _ => (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        event_type: SessionEventType::Delete,
                        data: None,
                        meta,
                        resource: Some(vec![resource]),
                        is_global: true,
                    }),
                ),
            },
            SessionAction::DataCreated(create_response) => {
                let resource = create_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let data_payload = create_response.data.clone();
                        let account: Rc<Account> =
                            Rc::new(serde_json::from_value(data_payload).unwrap());
                        let current_state_data = &store.data;
                        let mut account_vec = current_state_data.accounts.borrow_mut();
                        account_vec.push(account.clone());
                        (
                            SessionStore {
                                data: StoreData {
                                    accounts: current_state_data.accounts.clone(),
                                    storage_status: current_state_data.storage_status.clone(),
                                },
                                ..store.deref().clone()
                            }
                            .into(),
                            Some(SessionEvent {
                                event_type: SessionEventType::Create,
                                data: Some(serde_json::to_value(account).unwrap()),
                                meta,
                                resource: Some(vec![resource]),
                                is_global: true,
                            }),
                        )
                    }
                    _ => (
                        SessionStore {
                            ..store.deref().clone()
                        }
                        .into(),
                        Some(SessionEvent {
                            event_type: SessionEventType::Create,
                            data: None,
                            meta,
                            resource: Some(vec![resource]),
                            is_global: true,
                        }),
                    ),
                }
            }
            SessionAction::DataEdited(edit_response) => {
                let resource = edit_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let data_payload = edit_response.data.clone();
                        let account: Rc<Account> =
                            Rc::new(serde_json::from_value(data_payload).unwrap());
                        let current_state_data = &store.data;
                        let mut account_vec = current_state_data.accounts.borrow_mut();
                        // TODO make sure edit_response.id is same as account.id. it is not the
                        // same in the current implementation
                        // response.id is previous domain+username, but account.id is new
                        // domain+new username
                        let account_id = edit_response.id.clone();
                        let mut meta = meta.unwrap_or(json!({}));
                        meta.as_object_mut().unwrap().insert(
                            "id".to_owned(),
                            serde_json::to_value(account_id.clone()).unwrap(),
                        );
                        let meta = Some(meta);
                        let index = account_vec
                            .iter()
                            .position(|ac| account_id == ac.id)
                            .unwrap();
                        account_vec[index] = account.clone();
                        (
                            SessionStore {
                                data: StoreData {
                                    accounts: current_state_data.accounts.clone(),
                                    storage_status: current_state_data.storage_status.clone(),
                                },
                                ..store.deref().clone()
                            }
                            .into(),
                            Some(SessionEvent {
                                event_type: SessionEventType::Update,
                                data: Some(serde_json::to_value(account).unwrap()),
                                meta,
                                resource: Some(vec![resource]),
                                is_global: true,
                            }),
                        )
                    }
                    _ => (
                        SessionStore {
                            ..store.deref().clone()
                        }
                        .into(),
                        Some(SessionEvent {
                            event_type: SessionEventType::Create,
                            data: None,
                            meta,
                            resource: Some(vec![resource]),
                            is_global: true,
                        }),
                    ),
                }
            }
            SessionAction::DataFetched(fetch_response) => {
                let session_data = store.data.clone();
                let data = fetch_response.data;
                let mut meta = meta.unwrap_or(json!({}));
                let meta_obj = meta.as_object_mut().unwrap();
                let response_meta = fetch_response.meta.clone().unwrap_or(json!({}));
                let path = response_meta.get("path");
                let path = path.and_then(|v| v.as_str());
                if path.is_some() {
                    meta_obj.insert("path".to_owned(), path.unwrap().into());
                }
                let resource = fetch_response.resource.clone();
                if resource == Resource::Account {
                    let data_payload: Vec<Rc<Account>> = data
                        .as_array()
                        .unwrap_or(&vec![])
                        .into_iter()
                        .cloned()
                        .map(|val| Rc::new(serde_json::from_value(val).unwrap()))
                        .collect();
                    let session_event = {
                        match session_data.storage_status {
                            _ => Some(SessionEvent {
                                event_type: SessionEventType::Refreshed,
                                data: Some(data),
                                meta: Some(meta),
                                resource: Some(vec![resource]),
                                is_global: true,
                            }),
                        }
                    };
                    let current_state_data = store.data.clone();
                    let mut account_section = current_state_data.accounts.borrow_mut();
                    *account_section = data_payload;
                    (
                        SessionStore {
                            data: StoreData {
                                accounts: current_state_data.accounts.clone(),
                                storage_status: StorageStatus::Loaded,
                            },
                            ..store.deref().clone()
                        }
                        .into(),
                        session_event,
                    )
                } else {
                    (
                        SessionStore {
                            ..store.deref().clone()
                        }
                        .into(),
                        None,
                    )
                }
            }
            SessionAction::DataLoading(acknowledgement) => (
                SessionStore {
                    data: StoreData {
                        storage_status: StorageStatus::Loading(acknowledgement),
                        ..store.data.clone()
                    },
                    ..store.deref().clone()
                }
                .into(),
                None,
            ),
            SessionAction::DataCreationFailed(resource, _data, request) => {
                dbg!(&resource);
                dbg!(&_data);
                dbg!(&meta);
                let session_event = {
                    if let Some(request) = request {
                        let request_acknowledgement = request.get_acknowledgement();
                        if let Some(request_acknowledgement) = request_acknowledgement {
                            extension_port_name =
                                PORT_ID_MAP.lock().unwrap().remove(&request_acknowledgement)
                        }
                        let session_event = match resource {
                            Resource::Account => Some(SessionEvent {
                                event_type: SessionEventType::CreationFailed,
                                data: Some(_data),
                                meta,
                                resource: Some(vec![resource]),
                                is_global: false,
                            }),
                            _ => None,
                        };
                        session_event
                    } else {
                        None
                    }
                };
                (
                    SessionStore {
                        data: StoreData {
                            ..store.data.clone()
                        },
                        ..store.deref().clone()
                    }
                    .into(),
                    session_event,
                )
            }
            _ => (
                SessionStore {
                    ..store.deref().clone()
                }
                .into(),
                None,
            ),
        };
        dbg!(&session_event);
        if let Some(session_event) = session_event {
            if session_event.is_global {
                api::extension_api::broadcast_session_event(session_event.clone());
                if clear_ports {
                    let locked = EXTENSION_PORT.lock();
                    let mut extension_ports = locked.unwrap();
                    extension_ports.iter().for_each(|(_, port)| {
                        port.disconnect();
                    });
                    extension_ports.clear();
                    LISTENER_PORT.lock().unwrap().clear();
                    trace!("cleared all ports in service worker");
                }
            } else {
                if let Some(extension_port_name) = extension_port_name {
                    if let Some(extension_port) =
                        EXTENSION_PORT.lock().unwrap().get(&extension_port_name)
                    {
                        api::extension_api::whisper_session_event(session_event, extension_port);
                    }
                }
            }
        }
        session_store
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = SessionStore;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        if let Err(err) = Self::Store::save(state.as_ref(), StorageArea::Session) {
            println!("Error saving state to storage: {:?}", err);
        } else {
        }
    }
}

pub async fn _set_passphrase_async(passphrase: Option<String>, dispatch: Dispatch<SessionStore>) {
    dispatch
        .reduce_mut_future(|store| {
            Box::pin(async move {
                store.user.1 = passphrase.clone();
            })
        })
        .await;
}
