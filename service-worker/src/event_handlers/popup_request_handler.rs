use crate::store::SessionAction;
use crate::store::SessionActionWrapper;
use crate::store::SessionStore;
use crate::store::EXTENSION_PORT;
use crate::store::LISTENER_PORT;
use crate::store::PORT_ID_MAP;
use crate::store::REQUEST_MAP;
use crate::Resource;
use browser_rpass::types::Account;
use browser_rpass::types::StorageStatus;
use browser_rpass::util::*;
use log::info;
pub use wasm_bindgen;
use wasm_bindgen::JsValue;
pub use wasm_bindgen_futures;
use yewdux::dispatch::Dispatch;

use browser_rpass::request::*;
use browser_rpass::response::*;
use gloo_utils::format::JsValueSerdeExt;
use serde_json;
use serde_json::json;
use std::collections::HashMap;
use std::collections::HashSet;
use wasm_bindgen::prelude::Closure;

use crate::store::NATIVE_PORT;

pub fn handle_request_from_popup(request: RequestEnum, extension_port: Port, _native_port: Port) {
    let dispatch = Dispatch::<SessionStore>::new();
    let meta = Some(json!({"requester_port_id":extension_port.name()}));
    wasm_bindgen_futures::spawn_local({
        let mut request = request.clone();
        let acknowledgement = request.get_acknowledgement().unwrap().clone();
        let extension_port = extension_port.clone();
        if extension_port.name().is_empty() {
            let acknowledgement = acknowledgement.clone();
            extension_port.set_name(acknowledgement.clone());
            PORT_ID_MAP
                .lock()
                .unwrap()
                .insert(acknowledgement, extension_port.name());
        }
        async move {
            let native_port = NATIVE_PORT.lock().borrow().clone();
            let native_request_acknowledgement: String = {
                if let Some(ref acknowledgement) = request.get_acknowledgement() {
                    acknowledgement.clone()
                } else {
                    let acknowledgement = create_request_acknowledgement();
                    request.set_acknowledgement(acknowledgement.clone());
                    acknowledgement
                }
            };
            if let RequestEnum::Init(_init_request) = request {
                let dispatch = Dispatch::<SessionStore>::new();
                if dispatch.get().verified {
                    let mock_session_event = {
                        SessionEvent {
                            event_type: SessionEventType::Login,
                            data: Some(json!({"verified":true})),
                            meta,
                            resource: None,
                            is_global: false,
                        }
                    };
                    let message = MessageEnum::Message(RequestEnum::create_session_event_request(
                        Some(acknowledgement.clone()),
                        mock_session_event.clone(),
                        None,
                    ));
                    PORT_ID_MAP
                        .lock()
                        .unwrap()
                        .insert(acknowledgement.clone(), extension_port.name());
                    extension_port
                        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&message).unwrap());
                }
                LISTENER_PORT
                    .lock()
                    .unwrap()
                    .entry(Resource::Auth)
                    .and_modify(|v| {
                        v.insert(extension_port.name());
                    })
                    .or_insert({
                        let mut set = HashSet::new();
                        set.insert(extension_port.name());
                        set
                    });
            } else if let Some(passphrase) = dispatch.get().passphrase.clone() {
                let header_map = {
                    let mut map = HashMap::new();
                    map.insert("passphrase".to_owned(), passphrase.clone());
                    map
                };
                request.set_header(header_map);
                match request.clone() {
                    RequestEnum::Get(get_request) => match get_request.resource.clone() {
                        _ => {
                            todo!();
                        }
                    },
                    RequestEnum::Search(search_request) => match search_request.resource.clone() {
                        _ => {
                            todo!();
                        }
                    },
                    RequestEnum::Logout(_logout_request) => {
                        let dispatch = Dispatch::<SessionStore>::new();
                        dispatch.apply(SessionActionWrapper {
                            meta,
                            action: SessionAction::Logout,
                        });
                    }
                    RequestEnum::Create(create_request) => {
                        REQUEST_MAP
                            .lock()
                            .unwrap()
                            .insert(native_request_acknowledgement.clone(), request.clone());
                        PORT_ID_MAP
                            .lock()
                            .unwrap()
                            .insert(acknowledgement.clone(), extension_port.name());
                        native_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&create_request).unwrap(),
                        );
                    }
                    RequestEnum::Delete(delete_request) => {
                        REQUEST_MAP
                            .lock()
                            .unwrap()
                            .insert(native_request_acknowledgement.clone(), request.clone());
                        PORT_ID_MAP
                            .lock()
                            .unwrap()
                            .insert(acknowledgement.clone(), extension_port.name());
                        native_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&delete_request).unwrap(),
                        );
                    }
                    RequestEnum::Fetch(fetch_request) => {
                        let meta = Some(json!({"requester_port_id":extension_port.name()}));
                        let data = dispatch.get().data.clone();
                        let current_status = &dispatch.get().data.storage_status;
                        LISTENER_PORT
                            .lock()
                            .unwrap()
                            .entry(fetch_request.resource.clone())
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
                                    meta,
                                    action: SessionAction::DataLoading(
                                        request.get_acknowledgement(),
                                    ),
                                });
                                PORT_ID_MAP
                                    .lock()
                                    .unwrap()
                                    .insert(acknowledgement.clone(), extension_port.name());
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
                                let accounts = data.accounts.clone();
                                let mock_session_event = {
                                    SessionEvent {
                                        event_type: SessionEventType::Refreshed,
                                        data: Some(
                                            serde_json::to_value(
                                                accounts
                                                    .borrow()
                                                    .iter()
                                                    .map(|v| (**v).clone())
                                                    .collect::<Vec<Account>>(),
                                            )
                                            .unwrap(),
                                        ),
                                        meta,
                                        resource: Some(vec![resource]),
                                        is_global: true,
                                    }
                                };
                                let message = MessageEnum::Message(
                                    RequestEnum::create_session_event_request(
                                        None,
                                        mock_session_event.clone(),
                                        None,
                                    ),
                                );
                                extension_port.post_message(
                                    <JsValue as JsValueSerdeExt>::from_serde(&message).unwrap(),
                                );
                            }
                            StorageStatus::Error => {
                                dispatch.apply(SessionActionWrapper {
                                    meta,
                                    action: SessionAction::DataLoading(
                                        request.get_acknowledgement(),
                                    ),
                                });
                            }
                        }
                    }
                    _ => {
                        let error_response = ErrorResponse {
                            message: Some("resource not supported".to_owned()),
                            acknowledgement: request.get_acknowledgement(),
                            code: Some(ErrorCode::NotSupported),
                        };
                        extension_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&error_response).unwrap(),
                        );
                    }
                }
            } else if let RequestEnum::Login(login_request) = request.clone() {
                REQUEST_MAP
                    .lock()
                    .unwrap()
                    .insert(native_request_acknowledgement.clone(), request.clone());
                PORT_ID_MAP
                    .lock()
                    .unwrap()
                    .insert(acknowledgement.clone(), extension_port.name());
                native_port.post_message(
                    <JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap(),
                );
            } else {
                let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                    message: Some("passphrase is not a string".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotSupported),
                });
                extension_port.post_message(
                    <JsValue as JsValueSerdeExt>::from_serde(&error_response).unwrap(),
                );
            }
        }
    });
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        info!("popup connected. Port info: {:?}", port);
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
                let mut resource_with_no_listeners = Vec::new();
                for (resource, listeners) in port_map.iter_mut() {
                    listeners.remove(&port.name());
                    if listeners.is_empty() {
                        resource_with_no_listeners.push(resource.clone());
                    }
                }
                drop(port_map);
                resource_with_no_listeners.iter().for_each(|resource| {
                    LISTENER_PORT.lock().unwrap().remove(resource);
                });
                EXTENSION_PORT.lock().unwrap().remove(&port.name());
            })
            .into_js_value(),
        );
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                info!("extension msg recieved in servcie worker: {:?}", msg);
                wasm_bindgen_futures::spawn_local(async move {
                    let request: RequestEnum =
                        <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                    handle_request_from_popup(request, port, NATIVE_PORT.lock().borrow().clone());
                });
            }
        });
        port.on_message().add_listener(cb.into_js_value());
        ports.insert(port.name(), port);
    });
    return on_connect_with_popup_cb;
}