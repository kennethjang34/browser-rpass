use crate::api::extension_api::whisper_session_event;
use crate::store::SessionAction;
use crate::store::SessionActionWrapper;
use crate::store::SessionStore;
use crate::store::StoreData;
use crate::store::EXTENSION_PORT;
use crate::store::LISTENER_PORT;
use crate::store::PORT_ID_MAP;
use crate::store::REQUEST_MAP;
use browser_rpass::js_binding::extension_api::*;
use browser_rpass::types::Account;
use browser_rpass::types::StateStoreStatus;
use browser_rpass::types::StorageStatus;
use browser_rpass::util::*;
use log::*;
pub use wasm_bindgen;
use wasm_bindgen::JsValue;
pub use wasm_bindgen_futures;
use yewdux::dispatch::Dispatch;

use browser_rpass::request::*;
use browser_rpass::response::*;
use gloo_utils::format::JsValueSerdeExt;
use serde_json;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use wasm_bindgen::prelude::Closure;
use yewdux::mrc::Mrc;

use crate::store::NATIVE_PORT;

pub fn handle_request_from_popup(
    request: RequestEnum,
    extension_port: Port,
    //TODO use this native port instead of the global one
    //(the same port but not the best practice..)
    _native_port: Option<&Port>,
) {
    let dispatch = Dispatch::<SessionStore>::new();
    wasm_bindgen_futures::spawn_local({
        let mut request = request.clone();
        let session_event_acknowledgement = request.get_acknowledgement().unwrap().clone();
        let extension_port = extension_port.clone();
        if extension_port.name().is_empty() {
            let acknowledgement = session_event_acknowledgement.clone();
            extension_port.set_name(acknowledgement.clone());
            PORT_ID_MAP
                .lock()
                .unwrap()
                .insert(acknowledgement, extension_port.name());
        }
        async move {
            if let Some(native_port) = NATIVE_PORT.lock().borrow().as_ref() {
                let native_request_acknowledgement: String = {
                    if let Some(ref acknowledgement) = request.get_acknowledgement() {
                        acknowledgement.clone()
                    } else {
                        let acknowledgement = create_request_acknowledgement();
                        request.set_acknowledgement(acknowledgement.clone());
                        acknowledgement
                    }
                };
                if let RequestEnum::Init(init_request) = request {
                    let config = &init_request.config;
                    if config
                        .get(&DataFieldType::ContentScript)
                        .map(|v| v == "true")
                        .unwrap_or(false)
                    {
                        let stores = dispatch.get().stores.clone();
                        let mut data = HashMap::new();
                        if let Some(default_store_id) =
                            dispatch.get().default_store.clone().borrow().deref()
                        {
                            LISTENER_PORT
                                .lock()
                                .unwrap()
                                .entry(default_store_id.clone())
                                .and_modify(|v| {
                                    v.insert(extension_port.name());
                                })
                                .or_insert({
                                    let mut set = HashSet::new();
                                    set.insert(extension_port.name());
                                    set
                                });
                            if stores.borrow().get(default_store_id).is_none() {
                                data.insert(
                                    DataFieldType::DefaultStoreAvailable,
                                    serde_json::to_value(false).unwrap(),
                                );
                            } else {
                                data.insert(
                                    DataFieldType::DefaultStoreID,
                                    serde_json::to_value(default_store_id).unwrap(),
                                );
                                data.insert(
                                    DataFieldType::DefaultStoreAvailable,
                                    serde_json::to_value(true).unwrap(),
                                );
                            }
                        } else {
                            data.insert(
                                DataFieldType::DefaultStoreAvailable,
                                serde_json::to_value(false).unwrap(),
                            );
                        }
                        let mut payload = HashMap::new();
                        payload.insert(
                            DataFieldType::Data,
                            serde_json::to_value(data.clone()).unwrap(),
                        );
                        let session_event = SessionEvent {
                            store_id: init_request.get_store_id().or(data
                                .get(&DataFieldType::DefaultStoreID)
                                .map(|v| v.as_str().unwrap().to_owned())),
                            event_type: SessionEventType::Init,
                            detail: Some(payload),
                            resource: None,
                            notification_target: NotificationTarget::Port {
                                port_id: extension_port.name(),
                            },
                            acknowledgement: init_request.get_acknowledgement(),
                        };
                        whisper_session_event(session_event, &extension_port);
                    } else {
                        if let Some(store_id) = init_request.get_store_id() {
                            LISTENER_PORT
                                .lock()
                                .unwrap()
                                .entry(store_id)
                                .and_modify(|v| {
                                    v.insert(extension_port.name());
                                })
                                .or_insert({
                                    let mut set = HashSet::new();
                                    set.insert(extension_port.name());
                                    set
                                });
                        }
                        let current_status = dispatch.get().status.clone();
                        match current_status {
                            StateStoreStatus::Uninitialized => {
                                todo!();
                            }
                            StateStoreStatus::Loading(_acknowledgement) => {}
                            StateStoreStatus::Loaded | StateStoreStatus::Idle => {
                                let store_ids = dispatch
                                    .get()
                                    .stores
                                    .clone()
                                    .borrow()
                                    .keys()
                                    .cloned()
                                    .collect::<Vec<String>>();
                                let store_ids = serde_json::to_value(store_ids).unwrap();
                                let keys = dispatch.get().keys.borrow().clone();
                                let mut payload = HashMap::new();
                                payload.insert(
                                    DataFieldType::StoreIDList,
                                    serde_json::to_value(store_ids.clone()).unwrap(),
                                );
                                payload.insert(
                                    DataFieldType::Keys,
                                    serde_json::to_value(keys).unwrap(),
                                );
                                let session_event = SessionEvent {
                                    store_id: init_request.get_store_id(),
                                    event_type: SessionEventType::Init,
                                    detail: Some(payload),
                                    resource: None,
                                    notification_target: NotificationTarget::Port {
                                        port_id: extension_port.name(),
                                    },
                                    acknowledgement: init_request.get_acknowledgement(),
                                };
                                whisper_session_event(session_event, &extension_port);
                            }
                            StateStoreStatus::Error => {
                                todo!()
                            }
                        }
                    }
                } else {
                    let header = {
                        let map = HashMap::new();
                        map
                    };
                    request.set_header(header);
                    match request.clone() {
                        RequestEnum::Get(get_request) => match get_request.resource.clone() {
                            _ => {
                                todo!();
                            }
                        },
                        RequestEnum::Search(search_request) => {
                            match search_request.resource.clone() {
                                _ => {
                                    todo!();
                                }
                            }
                        }
                        RequestEnum::Logout(logout_request) => {
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&logout_request).unwrap(),
                            );
                        }
                        RequestEnum::Create(create_request) => {
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&create_request).unwrap(),
                            );
                        }
                        RequestEnum::CreateStore(create_store_request) => {
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&create_store_request)
                                    .unwrap(),
                            );
                        }
                        RequestEnum::DeleteStore(delete_store_request) => {
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&delete_store_request)
                                    .unwrap(),
                            );
                        }
                        RequestEnum::Edit(edit_request) => {
                            debug!("edit request: {:?}", edit_request);
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&edit_request).unwrap(),
                            );
                        }
                        RequestEnum::Delete(delete_request) => {
                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&delete_request).unwrap(),
                            );
                        }
                        RequestEnum::Fetch(fetch_request) => {
                            // let detail = Some(json!({"requester_port_id":extension_port.name()}));
                            let mut detail = HashMap::new();
                            detail.insert(DataFieldType::PortID, extension_port.name().into());
                            let store_state = dispatch.get().stores.clone();
                            let current_status = store_state
                                .borrow_mut()
                                .entry(fetch_request.store_id.clone().unwrap())
                                .or_insert({
                                    StoreData {
                                        accounts: Mrc::new(Vec::new()),
                                        storage_status: StorageStatus::Uninitialized,
                                        store_id: fetch_request.store_id.clone().unwrap(),
                                        signing_key: None,
                                        verified: false,
                                    }
                                })
                                .storage_status
                                .clone();
                            LISTENER_PORT
                                .lock()
                                .unwrap()
                                .entry(fetch_request.store_id.clone().unwrap())
                                .and_modify(|v| {
                                    v.insert(extension_port.name());
                                })
                                .or_insert({
                                    let mut set = HashSet::new();
                                    set.insert(extension_port.name());
                                    set
                                });
                            match current_status {
                                StorageStatus::Uninitialized => {
                                    dispatch.apply(SessionActionWrapper {
                                        detail: Some(detail),
                                        action: SessionAction::DataLoading(
                                            fetch_request.store_id.clone().unwrap(),
                                            request.get_acknowledgement(),
                                        ),
                                    });
                                    PORT_ID_MAP.lock().unwrap().insert(
                                        session_event_acknowledgement.clone(),
                                        extension_port.name(),
                                    );
                                    REQUEST_MAP.lock().unwrap().insert(
                                        native_request_acknowledgement.clone(),
                                        request.clone(),
                                    );
                                    native_port.post_message(
                                        <JsValue as JsValueSerdeExt>::from_serde(&fetch_request)
                                            .unwrap(),
                                    );
                                }
                                StorageStatus::Loading(_acknowledgement) => {}
                                StorageStatus::Loaded => {
                                    let resource = fetch_request.resource.clone();
                                    let store = dispatch.get().stores.clone();
                                    let accounts = store
                                        .borrow()
                                        .get(&fetch_request.store_id.clone().unwrap())
                                        .unwrap()
                                        .accounts
                                        .clone();
                                    let mut data = HashMap::new();
                                    data.insert(
                                        DataFieldType::Data,
                                        serde_json::to_value(
                                            accounts
                                                .borrow()
                                                .iter()
                                                .map(|v| (**v).clone())
                                                .collect::<Vec<Account>>(),
                                        )
                                        .unwrap(),
                                    );
                                    data.insert(
                                        DataFieldType::PortID,
                                        extension_port.name().into(),
                                    );
                                    let mock_session_event = {
                                        SessionEvent {
                                            store_id: fetch_request.store_id.clone(),
                                            event_type: SessionEventType::Refreshed,
                                            detail: Some(data),
                                            resource: Some(vec![resource]),
                                            notification_target: if let Some(store_id) =
                                                fetch_request.store_id.clone()
                                            {
                                                NotificationTarget::Store { store_id }
                                            } else {
                                                NotificationTarget::All
                                            },
                                            acknowledgement: Some(
                                                session_event_acknowledgement.clone(),
                                            ),
                                        }
                                    };
                                    let message = RequestEnum::create_session_event_request(
                                        mock_session_event.clone(),
                                    );
                                    extension_port.post_message(
                                        <JsValue as JsValueSerdeExt>::from_serde(&message).unwrap(),
                                    );
                                }
                                StorageStatus::Error => {
                                    todo!();
                                }
                            }
                        }
                        RequestEnum::Login(login_request) => {
                            LISTENER_PORT
                                .lock()
                                .unwrap()
                                .entry(login_request.store_id.clone().unwrap())
                                .and_modify(|v| {
                                    v.insert(extension_port.name());
                                })
                                .or_insert({
                                    let mut set = HashSet::new();
                                    set.insert(extension_port.name());
                                    set
                                });

                            REQUEST_MAP
                                .lock()
                                .unwrap()
                                .insert(native_request_acknowledgement.clone(), request.clone());
                            PORT_ID_MAP.lock().unwrap().insert(
                                session_event_acknowledgement.clone(),
                                extension_port.name(),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap(),
                            );
                        }
                        _ => {
                            error!("resouce not supported. received request: {:?}", request);
                            let error_response = ErrorResponse {
                                message: Some(
                                    format!(
                                        "resource not supported. received request: {:?}",
                                        request
                                    )
                                    .to_owned(),
                                ),
                                acknowledgement: request.get_acknowledgement(),
                                code: Some(ErrorCode::NotSupported),
                            };
                            extension_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&error_response).unwrap(),
                            );
                        }
                    }
                }
            } else {
                error!("native port not found");
                let session_event = SessionEvent {
                    store_id: None,
                    event_type: SessionEventType::NativeAppConnectionError,
                    detail: None,
                    resource: None,
                    notification_target: NotificationTarget::Port {
                        port_id: extension_port.name(),
                    },
                    acknowledgement: request.get_acknowledgement(),
                };
                whisper_session_event(session_event, &extension_port);
            }
        }
    });
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let native_port = NATIVE_PORT.lock().borrow().clone();
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        trace!("popup connected. Port info: {:?}", port);
        let mut ports = EXTENSION_PORT.lock().unwrap();
        if port.name().is_empty() {
            let acknowledgement = create_request_acknowledgement();
            port.set_name(acknowledgement.clone());
        }
        port.on_disconnect().add_listener(
            Closure::<dyn FnMut(Port)>::new(move |port: Port| {
                EXTENSION_PORT.lock().unwrap().remove(&port.name());
                let locked = LISTENER_PORT.lock();
                let mut port_map = locked.unwrap();
                let mut store_id_with_no_listeners = Vec::new();
                for (store_id, listeners) in port_map.iter_mut() {
                    listeners.remove(&port.name());
                    if listeners.is_empty() {
                        store_id_with_no_listeners.push(store_id.clone());
                    }
                }
                drop(port_map);
                store_id_with_no_listeners.iter().for_each(|store_id| {
                    LISTENER_PORT.lock().unwrap().remove(store_id);
                });
                EXTENSION_PORT.lock().unwrap().remove(&port.name());
            })
            .into_js_value(),
        );
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            let native_port = native_port.clone();
            move |msg: JsValue, port: Port| {
                let native_port = native_port.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let request: RequestEnum =
                        <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                    handle_request_from_popup(request, port, native_port.as_ref());
                });
            }
        });
        port.on_message().add_listener(cb.into_js_value());
        ports.insert(port.name(), port);
    });
    return on_connect_with_popup_cb;
}
