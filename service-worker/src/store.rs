use crate::event_handlers::native_message_handler::process_native_message;
pub use crate::Resource;
use crate::{api, StorageStatus};
use browser_rpass::dbg;
use browser_rpass::request::{LoginRequest, RequestEnumTrait, SessionEventType};
use browser_rpass::response::{CreateResponse, EditResponse, FetchResponse, LogoutResponse};
use browser_rpass::store;
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
use web_sys::console;
use yewdux::mrc::Mrc;

use yewdux::prelude::Reducer;

pub use browser_rpass::{
    js_binding::extension_api::*,
    request::{RequestEnum, SessionEvent, SessionEventWrapper},
    store::AsyncCallback,
    store::MESSAGE_ACKNOWLEDGEMENTS_NATIVE,
    store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP,
};
use gloo::storage::errors::StorageError;
use lazy_static::lazy_static;
use yewdux::{
    prelude::{init_listener, Listener},
    store::Store,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SessionAction {
    Login,
    LoginError(LoginRequest),
    Logout,
    LogoutError(LogoutResponse),
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
    match serde_json::from_slice::<Value>(&msg.as_bytes()) {
        Ok(parsed_json) => {
            let _ = process_native_message(parsed_json, NATIVE_PORT.lock().borrow().clone(), None);
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
        #[allow(unused_mut)]
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
    pub current_store_id: Option<String>,
    pub current_signing_key: Option<String>,
    pub verified: bool,
    pub data: StoreData,
}

impl Store for SessionStore {
    fn new() -> Self {
        init_listener(StorageListener);
        SessionStore::default()
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl SessionStore {
    pub async fn load() -> Option<SessionStore> {
        let loaded = chrome
            .storage()
            .session()
            .get_item(&type_name::<SessionStore>(), store::StorageArea::Session)
            .await;
        if let Ok(value) = loaded {
            let parsed = <JsValue as JsValueSerdeExt>::into_serde::<String>(&value);
            if let Ok(json_string) = parsed {
                let state = serde_json::from_str::<SessionStore>(&json_string);
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
                    let current = chrome
                        .storage()
                        .session()
                        .get_all(store::StorageArea::Session)
                        .await
                        .unwrap();
                    console::log_1(&current);
                }
            }
        });
        Ok(())
    }
}

impl Reducer<SessionStore> for SessionActionWrapper {
    fn apply(self, store: Rc<SessionStore>) -> Rc<SessionStore> {
        let meta = self.meta;
        let mut extension_port_name: Option<String> = None;
        let acknowledgement = meta
            .as_ref()
            .and_then(|meta| meta.get("acknowledgement"))
            .and_then(|ack| ack.as_str())
            .map(|ack| ack.to_owned());
        let session_action = self.action;
        let mut clear_ports = false;
        let (session_store, session_event) = match session_action {
            SessionAction::Login => {
                let store_id = {
                    if let Some(ref meta) = meta {
                        if let Some(store_id) = meta.get("store_id") {
                            Some(store_id.as_str().unwrap().to_owned())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };
                (
                    SessionStore {
                        current_store_id: store_id.clone(),
                        verified: true,
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        event_type: SessionEventType::Login,
                        data: Some(json!({"verified":true,"store_id":store_id.clone()})),
                        meta,
                        resource: Some(vec![Resource::Auth]),
                        is_global: true,
                        acknowledgement,
                    }),
                )
            }
            SessionAction::LoginError(request) => {
                let request_acknowledgement = request.get_acknowledgement();
                if let Some(ref request_acknowledgement) = request_acknowledgement {
                    extension_port_name =
                        PORT_ID_MAP.lock().unwrap().remove(request_acknowledgement);
                };

                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        event_type: SessionEventType::LoginError,
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Auth]),
                        is_global: false,
                        acknowledgement: request_acknowledgement,
                    }),
                )
            }
            SessionAction::Logout => {
                if (*store).verified {
                    clear_ports = true;
                    (
                        SessionStore {
                            verified: false,
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
                            acknowledgement,
                        }),
                    )
                } else {
                    (
                        store,
                        Some(SessionEvent {
                            event_type: SessionEventType::LogoutError,
                            data: None,
                            meta,
                            resource: Some(vec![Resource::Auth]),
                            is_global: false,
                            acknowledgement,
                        }),
                    )
                }
            }
            SessionAction::DataDeleted(resource, data) => match resource.clone() {
                Resource::Account => {
                    let map = data.as_object().unwrap();
                    let deleted_id = map.get("id").unwrap().as_str().unwrap().to_owned();
                    let mut account_vec = store.data.accounts.borrow_mut();
                    let index = account_vec.iter().position(|ac| deleted_id == ac.id);
                    if let Some(index) = index {
                        account_vec.remove(index);
                    } else {
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
                            acknowledgement,
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
                        acknowledgement,
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
                                acknowledgement,
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
                            acknowledgement,
                        }),
                    ),
                }
            }
            SessionAction::DataEdited(edit_response) => {
                let resource = edit_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let data_payload = edit_response.data.clone();
                        let updated_data = data_payload.as_object().unwrap();
                        let current_state_data = &store.data;
                        let mut account_vec = current_state_data.accounts.borrow_mut();
                        let account_id = edit_response.id.clone();
                        let mut meta = meta.unwrap_or(json!({}));
                        meta.as_object_mut().unwrap().insert(
                            "id".to_owned(),
                            serde_json::to_value(account_id.clone()).unwrap(),
                        );
                        let meta = Some(meta);
                        let account_idx = account_vec
                            .iter()
                            .position(|ac| ac.id == account_id)
                            .unwrap();
                        let account = account_vec.get_mut(account_idx).unwrap();
                        let new_account: &mut Account = Rc::make_mut(account);
                        for (key, value) in updated_data {
                            //TODO compare through each field's string value, rather than manually
                            //checking each field with string literal
                            if let Some(new_value) = value.get("new") {
                                match key.as_str() {
                                    "username" => {
                                        new_account.username =
                                            new_value.as_str().unwrap().to_owned();
                                    }
                                    "password" => {
                                        new_account.password =
                                            Some(new_value.as_str().unwrap().to_owned());
                                    }
                                    "domain" => {
                                        new_account.domain =
                                            Some(new_value.as_str().unwrap().to_owned());
                                    }
                                    "note" => {
                                        new_account.note =
                                            Some(new_value.as_str().unwrap().to_owned());
                                    }
                                    "path" => {
                                        new_account.path =
                                            Some(new_value.as_str().unwrap().to_owned());
                                    }
                                    _ => {}
                                }
                            }
                        }
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
                                data: Some(serde_json::to_value(new_account).unwrap()),
                                meta,
                                resource: Some(vec![resource]),
                                is_global: true,
                                acknowledgement,
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
                            acknowledgement,
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
                                acknowledgement,
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
                                acknowledgement,
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
        if let Err(err) = Self::Store::save(state.as_ref(), store::StorageArea::Session) {
            println!("Error saving state to storage: {:?}", err);
        } else {
        }
    }
}
