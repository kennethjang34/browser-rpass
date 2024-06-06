use crate::event_handlers::native_message_handler::process_native_message;
use crate::DataFieldType;
pub use crate::Resource;
use crate::{api, StorageStatus};
use browser_rpass::request::{
    LoginRequest, NotificationTarget, RequestEnumTrait, SessionEventType,
};
use browser_rpass::response::{
    CreateResponse, CreateStoreResponse, DeleteStoreResponse, EditResponse, FetchResponse,
    InitResponse, LogoutResponse, ResponseEnum,
};
use browser_rpass::store;
use browser_rpass::types::*;
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
use yewdux::dispatch::Dispatch;
use yewdux::mrc::Mrc;

use yewdux::prelude::Reducer;

pub use browser_rpass::{
    js_binding::extension_api::*,
    request::{RequestEnum, SessionEvent},
};
use gloo::storage::errors::StorageError;
use lazy_static::lazy_static;
use yewdux::{
    prelude::{init_listener, Listener},
    store::Store,
};
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SessionAction {
    Login {
        store_id: String,
        is_default: bool,
        context: Option<HashMap<DataFieldType, Value>>,
    },
    LoginError(LoginRequest),
    Logout(Option<String>, Option<String>),
    Init(InitResponse),
    InitStarted(RequestEnum),
    LogoutError(LogoutResponse),
    DataFetched(FetchResponse),
    DataLoading(String, Option<String>),
    DataCreated(CreateResponse),
    StoreCreated(CreateStoreResponse),
    StoreCreationFailed(RequestEnum, ResponseEnum),
    StoreDeleted(DeleteStoreResponse),
    StoreDeletionFailed(RequestEnum, ResponseEnum),
    DataEdited(EditResponse),
    DataDeleted(Resource, String, HashMap<DataFieldType, Value>),
    DataDeletionFailed(Resource, String),
    DataCreationFailed(Resource, HashMap<DataFieldType, Value>, Option<RequestEnum>),
    DataEditFailed(Resource, HashMap<DataFieldType, Value>, Option<RequestEnum>),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionActionWrapper {
    pub detail: Option<HashMap<DataFieldType, Value>>,
    pub action: SessionAction,
}

fn native_port_disconnect_handler(_port: Port) {
    {
        if let Ok(mut borrowed) = NATIVE_PORT.lock().try_borrow_mut() {
            *borrowed = None;
        }
    }
    // need to add async task to receive repeated disconnect events.
    // mostly to detect if native app is not configured properly
    wasm_bindgen_futures::spawn_local(async move {
        {
            if let Ok(borrowed) = NATIVE_PORT.lock().try_borrow() {
                if borrowed.is_none() {
                    error!(
                    "native port disconnected, but no port found. Likely native app not reachable."
                );
                    return;
                }
            }
        }
        let new_port = chrome.runtime().connect_native("rpass");
        new_port.on_disconnect().add_listener(
            Closure::<dyn Fn(Port)>::new(native_port_disconnect_handler).into_js_value(),
        );
        new_port.on_message().add_listener(
            Closure::<dyn Fn(String)>::new(native_port_message_handler).into_js_value(),
        );
        #[allow(unused_mut)]
        let mut init_config = HashMap::new();
        let init_request = RequestEnum::create_init_request(init_config, None, None);
        new_port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
        if let Ok(mut borrowed) = NATIVE_PORT.lock().try_borrow_mut() {
            *borrowed = Some(new_port);
        }
    });
}
fn native_port_message_handler(msg: String) {
    match serde_json::from_slice::<Value>(&msg.as_bytes()) {
        Ok(parsed_json) => {
            let _ = process_native_message(parsed_json, NATIVE_PORT.lock().borrow().as_ref(), None);
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
    pub static ref NATIVE_PORT: ReentrantMutex<RefCell<Option<Port>>> = {
        let port = chrome.runtime().connect_native("rpass");
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
        let dispatch = Dispatch::<SessionStore>::new();
        dispatch.apply(SessionActionWrapper {
            action: SessionAction::InitStarted(init_request),
            detail: None,
        });
        ReentrantMutex::new(RefCell::new(Some(port)))
    };
}
lazy_static! {
    pub static ref EXTENSION_PORT: Mutex<HashMap<String, Port>> = Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref LISTENER_PORT: Mutex<HashMap<String, HashSet<String>>> =
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
    pub signing_key: Option<String>,
    pub store_id: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SessionStore {
    pub stores: Mrc<HashMap<String, StoreData>>,
    pub keys: Mrc<Vec<Key>>,
    pub default_store: Mrc<Option<String>>,
    pub status: StateStoreStatus,
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
                    let _current = chrome
                        .storage()
                        .session()
                        .get_all(store::StorageArea::Session)
                        .await
                        .unwrap();
                }
            }
        });
        Ok(())
    }
}

impl Reducer<SessionStore> for SessionActionWrapper {
    fn apply(self, store: Rc<SessionStore>) -> Rc<SessionStore> {
        let detail = self.detail;
        let mut extension_port_name: Option<String> = None;
        let acknowledgement = detail
            .as_ref()
            .and_then(|meta| meta.get(&DataFieldType::Acknowledgement))
            .and_then(|ack| ack.as_str())
            .map(|ack| ack.to_owned());
        let session_action = self.action;
        #[allow(unused_mut)]
        let mut ports_to_disconnect = HashMap::<String, HashSet<String>>::new();
        let (session_store, session_event) = match session_action {
            SessionAction::Login {
                store_id,
                is_default,
                context,
            } => {
                let store_id = {
                    detail
                        .as_ref()
                        .unwrap()
                        .get(&DataFieldType::StoreID)
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned()
                };
                let is_default = {
                    detail
                        .as_ref()
                        .unwrap()
                        .get(&DataFieldType::IsDefault)
                        .unwrap()
                        .as_bool()
                        .unwrap()
                };
                let mut data = HashMap::new();
                data.insert(DataFieldType::Verified, json!(true));
                data.insert(DataFieldType::StoreID, json!(store_id.clone()));
                data.insert(DataFieldType::IsDefault, json!(is_default));
                let mut stores_ptr = store.stores.borrow_mut();
                if let Some(store) = stores_ptr.get_mut(&store_id.clone()) {
                    store.verified = true;
                } else {
                    stores_ptr.insert(
                        store_id.clone(),
                        StoreData {
                            accounts: Mrc::new(Vec::new()),
                            storage_status: StorageStatus::Uninitialized,
                            signing_key: None,
                            store_id: store_id.clone(),
                            verified: true,
                        },
                    );
                }
                if is_default {
                    store.default_store.borrow_mut().replace(store_id.clone());
                }
                if let Some(prev_store_id) =
                    detail.as_ref().unwrap().get(&DataFieldType::PrevStoreID)
                {
                    if let Some(prev_store_id) = prev_store_id.as_str().to_owned() {
                        if prev_store_id != store_id {
                            let mut locked = LISTENER_PORT.lock().unwrap();
                            if let Some(store) = locked.get_mut(prev_store_id) {
                                ports_to_disconnect
                                    .entry(prev_store_id.to_string())
                                    .and_modify(|v| {
                                        v.extend(store.clone());
                                    })
                                    .or_insert_with(|| store.clone());
                            }
                        }
                    }
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        event_type: SessionEventType::Login,
                        store_id: Some(store_id.clone()),
                        detail: Some(data),
                        resource: Some(vec![Resource::Auth]),
                        notification_target: NotificationTarget::Store { store_id },
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
                        store_id: request.store_id,
                        event_type: SessionEventType::LoginError,
                        detail: None,
                        resource: Some(vec![Resource::Auth]),
                        notification_target: {
                            if let Some(port_id) = extension_port_name.clone() {
                                NotificationTarget::Port { port_id }
                            } else {
                                NotificationTarget::None
                            }
                        },
                        acknowledgement: request_acknowledgement,
                    }),
                )
            }
            SessionAction::Logout(store_id, _acknowledgement_opt) => {
                if let Some(store_id) = store_id.clone() {
                    if let Some(store) = LISTENER_PORT.lock().unwrap().get_mut(&store_id) {
                        ports_to_disconnect
                            .entry(store_id.clone())
                            .and_modify(|v| {
                                v.extend(store.clone());
                            })
                            .or_insert_with(|| store.clone());
                    }

                    if let Some(target_store) = store.stores.clone().borrow_mut().get_mut(&store_id)
                    {
                        if (*target_store).verified {
                            let mut data = HashMap::new();
                            data.insert(DataFieldType::StoreID, json!(store_id));
                            target_store.storage_status = StorageStatus::Uninitialized;
                            target_store.verified = false;
                            (
                                store,
                                Some(SessionEvent {
                                    store_id: Some(store_id.clone()),
                                    event_type: SessionEventType::Logout,
                                    detail: Some(data),
                                    resource: Some(vec![Resource::Auth]),
                                    notification_target: NotificationTarget::Store { store_id },
                                    acknowledgement,
                                }),
                            )
                        } else {
                            (
                                store,
                                Some(SessionEvent {
                                    store_id: Some(store_id),
                                    event_type: SessionEventType::LogoutError,
                                    detail: None,
                                    resource: Some(vec![Resource::Auth]),
                                    notification_target: if let Some(port_id) =
                                        extension_port_name.clone()
                                    {
                                        NotificationTarget::Port { port_id }
                                    } else {
                                        NotificationTarget::None
                                    },
                                    acknowledgement,
                                }),
                            )
                        }
                    } else {
                        (store, None)
                    }
                } else {
                    store.stores.borrow_mut().values_mut().for_each(|store| {
                        (*store) = StoreData {
                            store_id: store.store_id.clone(),
                            ..Default::default()
                        }
                    });
                    (
                        store,
                        Some(SessionEvent {
                            store_id: None,
                            event_type: SessionEventType::Logout,
                            detail: None,
                            resource: Some(vec![Resource::Auth]),
                            notification_target: NotificationTarget::All,
                            acknowledgement,
                        }),
                    )
                }
            }
            SessionAction::DataDeleted(resource, resource_id, data) => match resource.clone() {
                Resource::Account => {
                    let mut store_id = None;
                    let mut stores_ptr = store.stores.borrow_mut().clone();
                    for store_ptr in stores_ptr.values_mut() {
                        let account_idx = store_ptr
                            .accounts
                            .borrow()
                            .iter()
                            .position(|ac| ac.id == resource_id);
                        if let Some(account_idx) = account_idx {
                            store_ptr.accounts.borrow_mut().remove(account_idx);
                            store_id = Some(store_ptr.store_id.clone());
                            break;
                        }
                    }
                    (
                        SessionStore {
                            ..store.deref().clone()
                        }
                        .into(),
                        Some(SessionEvent {
                            store_id: store_id.clone(),
                            event_type: SessionEventType::Delete,
                            detail: Some(data),
                            resource: Some(vec![resource]),
                            notification_target: if let Some(store_id) = store_id.clone() {
                                NotificationTarget::Store { store_id }
                            } else {
                                // This should not happen.
                                NotificationTarget::All
                            },
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
                        store_id: None,
                        event_type: SessionEventType::Delete,
                        detail: None,
                        resource: Some(vec![resource]),
                        notification_target: NotificationTarget::All,
                        acknowledgement,
                    }),
                ),
            },
            SessionAction::DataCreated(mut create_response) => {
                let resource = create_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let data_payload =
                            create_response.detail.remove(&DataFieldType::Data).unwrap();
                        let account: Rc<Account> =
                            Rc::new(serde_json::from_value(data_payload).unwrap());
                        let mut stores_ptr = store.stores.borrow_mut().clone();
                        let mut data = HashMap::new();
                        if let Some(store_updated) = stores_ptr.get_mut(&create_response.store_id) {
                            let mut account_vec = store_updated.accounts.borrow_mut();
                            account_vec.push(account.clone());
                            data.insert(
                                DataFieldType::Data,
                                serde_json::to_value(account).unwrap(),
                            );
                        }
                        (
                            SessionStore {
                                ..store.deref().clone()
                            }
                            .into(),
                            Some(SessionEvent {
                                store_id: Some(create_response.store_id.clone()),
                                event_type: SessionEventType::Create,
                                detail: Some(data),
                                resource: Some(vec![resource]),
                                notification_target: NotificationTarget::Store {
                                    store_id: create_response.store_id,
                                },
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
                            store_id: None,
                            event_type: SessionEventType::Create,
                            detail: None,
                            resource: Some(vec![resource]),
                            notification_target: NotificationTarget::All,
                            acknowledgement,
                        }),
                    ),
                }
            }
            SessionAction::DataEdited(edit_response) => {
                let resource = edit_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let update_logs = edit_response.update_logs;
                        let mut stores_ptr = store.stores.borrow_mut().clone();
                        let mut detail = HashMap::new();
                        for store_ptr in stores_ptr.values_mut() {
                            if let Some(account) = store_ptr
                                .accounts
                                .borrow_mut()
                                .iter_mut()
                                .find(|ac| ac.id == edit_response.instance_id)
                            {
                                detail.insert(
                                    DataFieldType::ResourceID,
                                    serde_json::to_value(account.id.clone()).unwrap(),
                                );
                                let new_account: &mut Account = Rc::make_mut(account);
                                for update_log in update_logs.iter() {
                                    let field = update_log.field.clone();
                                    let new = update_log.new.clone();
                                    match field {
                                        DataFieldType::Username => {
                                            new_account.username = new.to_string()
                                        }
                                        DataFieldType::Password => {
                                            new_account.set_password(Some(new.to_string()))
                                        }
                                        DataFieldType::Domain => {
                                            new_account.domain = Some(new.to_string())
                                        }
                                        DataFieldType::Note => {
                                            new_account.note = Some(new.to_string())
                                        }
                                        _ => {}
                                    }
                                }
                                detail.insert(
                                    DataFieldType::Data,
                                    serde_json::to_value(new_account).unwrap(),
                                );
                                break;
                            }
                        }
                        (
                            SessionStore {
                                ..store.deref().clone()
                            }
                            .into(),
                            Some(SessionEvent {
                                store_id: Some(edit_response.store_id.clone()),
                                event_type: SessionEventType::Update,
                                detail: Some(detail),
                                resource: Some(vec![resource]),
                                notification_target: NotificationTarget::Store {
                                    store_id: edit_response.store_id,
                                },
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
                            store_id: Some(edit_response.store_id.clone()),
                            event_type: SessionEventType::Create,
                            detail: None,
                            resource: Some(vec![resource]),
                            notification_target: NotificationTarget::Store {
                                store_id: edit_response.store_id,
                            },
                            acknowledgement,
                        }),
                    ),
                }
            }
            SessionAction::DataFetched(fetch_response) => {
                let mut stores_ptr = store.stores.borrow_mut();
                let session_data = stores_ptr.get_mut(&fetch_response.store_id);
                if let Some(session_data) = session_data {
                    let detail = fetch_response.detail;
                    let resource = fetch_response.resource.clone();
                    if resource == Resource::Account {
                        let data_payload: Vec<Rc<Account>> = detail
                            .get(&DataFieldType::Data)
                            .unwrap_or(&json!([]))
                            .as_array()
                            .unwrap_or(&vec![])
                            .into_iter()
                            .cloned()
                            .map(|val| Rc::new(serde_json::from_value(val).unwrap()))
                            .collect();
                        session_data.storage_status = StorageStatus::Loaded;
                        session_data.verified = true;
                        let session_event = {
                            match session_data.storage_status {
                                _ => Some(SessionEvent {
                                    store_id: Some(fetch_response.store_id.clone()),
                                    event_type: SessionEventType::Refreshed,
                                    detail: Some(detail),
                                    resource: Some(vec![resource]),
                                    notification_target: NotificationTarget::Store {
                                        store_id: fetch_response.store_id,
                                    },
                                    acknowledgement,
                                }),
                            }
                        };
                        let mut account_section = session_data.accounts.borrow_mut();
                        *account_section = data_payload;
                        (
                            SessionStore {
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
            SessionAction::Init(init_response) => {
                let mut stores_ptr = store.stores.borrow_mut().clone();
                let store_ids = init_response
                    .detail
                    .get(&DataFieldType::StoreIDList)
                    .cloned()
                    .unwrap_or(json!([]))
                    .as_array()
                    .cloned()
                    .unwrap_or(vec![]);
                let raw_keys = init_response
                    .detail
                    .get(&DataFieldType::Keys)
                    .cloned()
                    .unwrap_or(json!([]))
                    .as_array()
                    .cloned()
                    .unwrap_or(vec![]);
                let keys: Vec<Key> = raw_keys.clone().into_iter().map(|key| key.into()).collect();
                for store_id in store_ids.iter() {
                    let store_id = store_id.as_str().unwrap().to_owned();
                    if stores_ptr.get(&store_id).is_none() {
                        stores_ptr.insert(
                            store_id.clone(),
                            StoreData {
                                accounts: Mrc::new(Vec::new()),
                                storage_status: StorageStatus::Uninitialized,
                                store_id,
                                signing_key: None,
                                verified: false,
                            },
                        );
                    }
                }
                let mut data = HashMap::new();
                data.insert(
                    DataFieldType::StoreIDList,
                    serde_json::to_value(store_ids).unwrap(),
                );
                data.insert(
                    DataFieldType::Keys,
                    serde_json::to_value(keys.clone()).unwrap(),
                );
                (
                    SessionStore {
                        stores: Mrc::new(stores_ptr),
                        status: StateStoreStatus::Loaded,
                        keys: Mrc::new(keys),
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id: None,
                        detail: Some(data.clone()),
                        event_type: SessionEventType::Init,
                        resource: Some(vec![Resource::Store]),
                        notification_target: NotificationTarget::All,
                        acknowledgement,
                    }),
                )
            }
            SessionAction::InitStarted(request) => {
                let request_acknowledgement = request.get_acknowledgement();
                (
                    SessionStore {
                        status: StateStoreStatus::Loading(request_acknowledgement.clone()),
                        ..store.deref().clone()
                    }
                    .into(),
                    None,
                )
            }
            SessionAction::DataLoading(store_id, acknowledgement) => {
                let mut stores_ptr = store.stores.borrow_mut().clone();
                if let Some(session_data) = stores_ptr.get_mut(&store_id) {
                    session_data.storage_status = StorageStatus::Loading(acknowledgement.clone());
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    None,
                )
            }
            SessionAction::DataCreationFailed(resource, _data, request) => {
                let session_event = {
                    if let Some(request) = request {
                        let request_acknowledgement = request.get_acknowledgement();
                        if let Some(request_acknowledgement) = request_acknowledgement {
                            extension_port_name =
                                PORT_ID_MAP.lock().unwrap().remove(&request_acknowledgement)
                        }
                        let session_event = match resource {
                            Resource::Account => Some(SessionEvent {
                                store_id: request.get_store_id(),
                                event_type: SessionEventType::CreationFailed,
                                detail: Some(_data),
                                resource: Some(vec![resource]),
                                notification_target: if let Some(port_id) =
                                    extension_port_name.clone()
                                {
                                    NotificationTarget::Port { port_id }
                                } else {
                                    NotificationTarget::None
                                },
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
                        ..store.deref().clone()
                    }
                    .into(),
                    session_event,
                )
            }
            SessionAction::StoreCreated(response) => {
                let store_id = response.store_id.clone();
                let mut stores_ptr = store.stores.borrow_mut();
                extension_port_name = PORT_ID_MAP
                    .lock()
                    .unwrap()
                    .remove(&response.acknowledgement.clone().unwrap());
                stores_ptr.insert(
                    store_id.clone(),
                    StoreData {
                        storage_status: StorageStatus::Uninitialized,
                        store_id: store_id.clone(),
                        ..Default::default()
                    },
                );
                let mut data = HashMap::new();
                data.insert(DataFieldType::StoreID, json!(store_id.clone()));
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id: Some(store_id.clone()),
                        event_type: SessionEventType::StoreCreated,
                        detail: Some(data),
                        resource: Some(vec![Resource::Store]),
                        notification_target: NotificationTarget::All,
                        acknowledgement,
                    }),
                )
            }
            SessionAction::StoreDeleted(response) => {
                let store_id = response.store_id.clone();
                let mut stores_ptr = store.stores.borrow_mut();
                extension_port_name = PORT_ID_MAP
                    .lock()
                    .unwrap()
                    .remove(&response.acknowledgement.clone().unwrap());
                stores_ptr.remove(&store_id);
                let mut detail = HashMap::new();
                detail.insert(DataFieldType::StoreID, json!(store_id.clone()));
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id: Some(store_id.clone()),
                        event_type: SessionEventType::StoreDeleted,
                        detail: Some(detail),
                        resource: Some(vec![Resource::Store]),
                        notification_target: NotificationTarget::All,
                        acknowledgement,
                    }),
                )
            }
            SessionAction::StoreDeletionFailed(request, response) => {
                let request_acknowledgement = request.get_acknowledgement();
                if let Some(request_acknowledgement) = request_acknowledgement {
                    extension_port_name =
                        PORT_ID_MAP.lock().unwrap().remove(&request_acknowledgement)
                }
                let mut data = HashMap::new();
                let store_id = request.get_store_id().unwrap_or_default();
                if let ResponseEnum::DeleteStoreResponse(response) = response {
                    data.insert(DataFieldType::StoreID, json!(store_id));
                    data.insert(DataFieldType::Error, json!(response.detail));
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id: Some(store_id.clone()),
                        event_type: SessionEventType::StoreDeletionFailed,
                        detail: None,
                        resource: Some(vec![Resource::Store]),
                        notification_target: if let Some(port_id) = extension_port_name.clone() {
                            NotificationTarget::Port { port_id }
                        } else {
                            NotificationTarget::None
                        },
                        acknowledgement,
                    }),
                )
            }
            SessionAction::StoreCreationFailed(request, response) => {
                let request_acknowledgement = request.get_acknowledgement();
                if let Some(request_acknowledgement) = request_acknowledgement {
                    extension_port_name =
                        PORT_ID_MAP.lock().unwrap().remove(&request_acknowledgement)
                }
                let mut data = HashMap::new();
                let store_id = request.get_store_id().unwrap_or_default();
                if let ResponseEnum::CreateResponse(response) = response {
                    data.insert(DataFieldType::StoreID, json!(store_id));
                    data.insert(DataFieldType::Error, json!(response.detail));
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id: Some(store_id.clone()),
                        event_type: SessionEventType::StoreCreationFailed,
                        detail: None,
                        resource: Some(vec![Resource::Store]),
                        notification_target: if let Some(port_id) = extension_port_name.clone() {
                            NotificationTarget::Port { port_id }
                        } else {
                            NotificationTarget::None
                        },
                        acknowledgement,
                    }),
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
            api::extension_api::broadcast_session_event(
                session_event.clone(),
                Some(ports_to_disconnect.clone()),
            );
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
