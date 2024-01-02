use std::collections::HashMap;

use browser_rpass::{
    js_binding::extension_api::Port,
    request::{DataFieldType, SessionEventType},
    response::{MessageEnum, RequestEnum, ResponseEnumTrait},
    store::{MESSAGE_ACKNOWLEDGEMENTS_POP_UP, MESSAGE_CONTEXT_POPUP},
    types::Resource,
};
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use serde_json::json;
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen_futures::spawn_local;
use yewdux::prelude::Dispatch;

use crate::{
    api::extension_api::fetch_accounts,
    store::{DataAction, LoginAction, LoginStatus, PopupStore},
};
pub fn create_message_listener(port: &Port) -> Closure<dyn Fn(JsValue)> {
    let port = port.clone();
    Closure::<dyn Fn(JsValue)>::new(move |msg: JsValue| {
        match <JsValue as JsValueSerdeExt>::into_serde::<MessageEnum>(&msg) {
            Ok(parsed_message) => match parsed_message {
                MessageEnum::Response(response) => {
                    let acknowledgement = response.get_acknowledgement().unwrap();
                    if let Some(request_cb) = MESSAGE_ACKNOWLEDGEMENTS_POP_UP
                        .lock()
                        .unwrap()
                        .remove(&acknowledgement)
                    {
                        let port2 = port.clone();
                        spawn_local(async move {
                            request_cb(response, port2.clone()).await;
                        });
                    } else {
                    }
                }
                MessageEnum::Message(request) => match request.clone() {
                    RequestEnum::SessionEventRequest(request) => {
                        let dispatch = Dispatch::<PopupStore>::new();
                        let event_request = request.session_event.clone();
                        let event_type = &event_request.event_type;
                        let data = event_request.data.clone().unwrap_or(HashMap::new());
                        let meta = request.session_event.clone().meta.unwrap_or(json!({}));
                        let contexts = MESSAGE_CONTEXT_POPUP.lock().unwrap();

                        let resource = event_request.resource.unwrap_or(vec![]);
                        match event_type {
                            &SessionEventType::Login => {
                                let store_id = data
                                    .get(&DataFieldType::StoreID)
                                    .map(|v| v.as_str().unwrap().to_owned())
                                    .unwrap();
                                if let LoginStatus::LoginStarted(current_store_id) =
                                    dispatch.get().login_status.clone()
                                {
                                    if current_store_id == store_id {
                                        dispatch.apply(LoginAction::LoginSucceeded(data));
                                        fetch_accounts(Some(store_id.clone()), None);
                                    }
                                } else {
                                    dispatch.apply(LoginAction::Login(store_id.clone(), data));
                                }
                            }
                            &SessionEventType::LoginError => {
                                let context = contexts
                                    .get(event_request.acknowledgement.as_ref().unwrap())
                                    .unwrap();
                                let store_id = context
                                    .get("store_id")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_string();
                                dispatch.apply(LoginAction::LoginError(data, store_id));
                            }
                            &SessionEventType::Logout => {
                                let store_id = data
                                    .get(&DataFieldType::StoreID)
                                    .map(|v| v.as_str().unwrap().to_owned())
                                    .unwrap();
                                dispatch.apply(LoginAction::Logout(store_id, data));
                            }
                            &SessionEventType::Delete => {
                                let resource = resource[0].clone();
                                match resource {
                                    Resource::Account => {
                                        dispatch.apply(DataAction::ResourceDeleted(resource, data));
                                    }
                                    _ => {}
                                }
                            }
                            &SessionEventType::StoreDeleted(ref data, ref store_id) => {
                                dispatch.apply(DataAction::StoreDeleted(
                                    data.clone(),
                                    store_id.clone(),
                                ));
                            }
                            &SessionEventType::StoreDeletionFailed(ref data, ref store_id) => {
                                dispatch.apply(DataAction::StoreDeletionFailed(
                                    data.clone(),
                                    store_id.clone(),
                                ));
                            }
                            &SessionEventType::Create => {
                                let resource = resource[0].clone();
                                match resource {
                                    Resource::Account => {
                                        dispatch.apply(DataAction::ResourceCreated(
                                            resource,
                                            data.clone().into(),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            &SessionEventType::Update => {
                                let resource = resource[0].clone();
                                match resource {
                                    Resource::Account => {
                                        dispatch.apply(DataAction::ResourceEdited(
                                            resource,
                                            data.clone().into(),
                                            meta.get("id").unwrap().as_str().unwrap().to_string(),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            &SessionEventType::CreationFailed => {
                                let resource = resource[0].clone();
                                match resource {
                                    Resource::Account => {
                                        dispatch.apply(DataAction::ResourceCreationFailed(
                                            resource, request,
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                            &SessionEventType::StoreCreated(ref data, ref store_id) => {
                                dispatch.apply(DataAction::StoreCreated(
                                    data.clone(),
                                    store_id.clone(),
                                ));
                            }
                            &SessionEventType::StoreCreationFailed(ref data, ref store_id) => {
                                dispatch.apply(DataAction::StoreCreationFailed(
                                    data.clone(),
                                    store_id.clone(),
                                ));
                            }
                            &SessionEventType::Refreshed => match resource[0] {
                                Resource::Account => match &dispatch.get().data.storage_status {
                                    _ => {
                                        dispatch.apply(DataAction::ResourceFetched(
                                            Resource::Account,
                                            data.clone().into(),
                                            None,
                                        ));
                                    }
                                },
                                _ => {}
                            },
                            &SessionEventType::Init(ref data) => {
                                let store = dispatch.get();
                                dispatch.apply(DataAction::Init(data.clone()));
                                if let Some(store_id) = store.persistent_data.store_id.as_ref() {
                                    if store.persistent_data.store_activated {
                                        fetch_accounts(Some(store_id.to_string()), None);
                                    }
                                }
                            }
                            _ => {
                                error!("unhandled event: {:?}", event_type);
                            }
                        }
                    }
                    _ => {
                        error!("unhandled request: {:?}", request);
                    }
                },
            },
            Err(e) => {
                error!(
                    "error happend while parsing:{:?}. Error message: {:?}",
                    msg, e
                );
            }
        }
    })
}
