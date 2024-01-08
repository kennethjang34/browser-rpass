use crate::event_handlers::native_message_handler::process_native_message;
pub use crate::Resource;
use crate::{api, remove_request_metadata, StorageStatus};
use browser_rpass::request::{DataFieldType, LoginRequest, RequestEnumTrait, SessionEventType};
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
    pub meta: Option<Value>,
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
            meta: None,
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
        let meta = self.meta;
        let mut extension_port_name: Option<String> = None;
        let acknowledgement = meta
            .as_ref()
            .and_then(|meta| meta.get("acknowledgement"))
            .and_then(|ack| ack.as_str())
            .map(|ack| ack.to_owned());
        let session_action = self.action;
        #[allow(unused_mut)]
        let mut ports_to_disconnect = HashMap::<String, HashSet<String>>::new();
        let (session_store, session_event) = match session_action {
            SessionAction::Login => {
                let store_id = {
                    meta.as_ref()
                        .unwrap()
                        .get("store_id")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_owned()
                };
                let is_default = {
                    meta.as_ref()
                        .unwrap()
                        .get("is_default")
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
                if let Some(prev_store_id) = meta.as_ref().unwrap().get("prev_store_id") {
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
                        store_id_index: Some(store_id),
                        data: Some(data),
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
                        store_id_index: request.store_id,
                        event_type: SessionEventType::LoginError,
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Auth]),
                        is_global: false,
                        acknowledgement: request_acknowledgement,
                    }),
                )
            }
            SessionAction::Logout(store_id, _acknowledgement_opt) => {
                if let Some(store_id) = store_id {
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
                                    store_id_index: Some(store_id),
                                    event_type: SessionEventType::Logout,
                                    data: Some(data),
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
                                    store_id_index: Some(store_id),
                                    event_type: SessionEventType::LogoutError,
                                    data: None,
                                    meta,
                                    resource: Some(vec![Resource::Auth]),
                                    is_global: false,
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
                            store_id_index: None,
                            event_type: SessionEventType::Logout,
                            data: None,
                            meta,
                            resource: Some(vec![Resource::Auth]),
                            is_global: true,
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
                            store_id_index: store_id,
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
                        store_id_index: None,
                        event_type: SessionEventType::Delete,
                        data: None,
                        meta,
                        resource: Some(vec![resource]),
                        is_global: true,
                        acknowledgement,
                    }),
                ),
            },
            SessionAction::DataCreated(mut create_response) => {
                let resource = create_response.resource.clone();
                match resource {
                    Resource::Account => {
                        let data_payload =
                            create_response.data.remove(&DataFieldType::Data).unwrap();
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
                                store_id_index: Some(create_response.store_id),
                                event_type: SessionEventType::Create,
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
                            store_id_index: None,
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
                        let updated_data = data_payload.get(&DataFieldType::UpdatedFields).unwrap();
                        let updated_data = updated_data.as_object().unwrap();
                        let mut stores_ptr = store.stores.borrow_mut().clone();
                        let mut data = HashMap::new();
                        let mut meta = meta.unwrap_or(json!({}));
                        for store_ptr in stores_ptr.values_mut() {
                            if let Some(account) = store_ptr
                                .accounts
                                .borrow_mut()
                                .iter_mut()
                                .find(|ac| ac.id == edit_response.id)
                            {
                                meta.as_object_mut().unwrap().insert(
                                    "id".to_owned(),
                                    serde_json::to_value(account.id.clone()).unwrap(),
                                );
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
                                            "password" => new_account.set_password(Some(
                                                new_value.as_str().unwrap().to_owned(),
                                            )),
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
                                data.insert(
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
                                store_id_index: Some(edit_response.store_id),
                                event_type: SessionEventType::Update,
                                data: Some(data),
                                meta: Some(meta),
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
                            store_id_index: Some(edit_response.store_id),
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
                let mut stores_ptr = store.stores.borrow_mut();
                let session_data = stores_ptr.get_mut(&fetch_response.store_id);
                if let Some(session_data) = session_data {
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
                                    store_id_index: Some(fetch_response.store_id),
                                    event_type: SessionEventType::Refreshed,
                                    data: Some(data),
                                    meta: Some(meta),
                                    resource: Some(vec![resource]),
                                    is_global: true,
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
                    .data
                    .get(&DataFieldType::StoreIDList)
                    .cloned()
                    .unwrap_or(json!([]))
                    .as_array()
                    .cloned()
                    .unwrap_or(vec![]);
                let raw_keys = init_response
                    .data
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
                        store_id_index: None,
                        event_type: SessionEventType::Init(data),
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Store]),
                        is_global: true,
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
                                store_id_index: request.get_store_id(),
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
                        //TODO: adding store id here causes error (only those subscribed to store id will receive the event, but the store just got created!)
                        store_id_index: None,
                        event_type: SessionEventType::StoreCreated(data, store_id),
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Store]),
                        is_global: true,
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
                let mut data = HashMap::new();
                data.insert(DataFieldType::StoreID, json!(store_id.clone()));
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id_index: None,
                        event_type: SessionEventType::StoreDeleted(data, store_id),
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Store]),
                        is_global: true,
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
                    data.insert(DataFieldType::Error, json!(response.data));
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id_index: Some(store_id.clone()),
                        event_type: SessionEventType::StoreDeletionFailed(data, store_id),
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Store]),
                        is_global: false,
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
                    data.insert(DataFieldType::Error, json!(response.data));
                }
                (
                    SessionStore {
                        ..store.deref().clone()
                    }
                    .into(),
                    Some(SessionEvent {
                        store_id_index: Some(store_id.clone()),
                        event_type: SessionEventType::StoreCreationFailed(data, store_id),
                        data: None,
                        meta,
                        resource: Some(vec![Resource::Store]),
                        is_global: false,
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
            if session_event.is_global {
                api::extension_api::broadcast_session_event(
                    session_event.clone(),
                    Some(ports_to_disconnect.clone()),
                );
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
