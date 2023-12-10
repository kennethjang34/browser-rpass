use browser_rpass::{
    js_binding::extension_api::Port,
    request::SessionEventType,
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

use crate::store::{DataAction, LoginAction, PopupStore};
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
                        let data = event_request.data.clone().unwrap_or(json!({}));
                        let meta = request.session_event.clone().meta.unwrap_or(json!({}));
                        let contexts = MESSAGE_CONTEXT_POPUP.lock().unwrap();

                        let resource = event_request.resource.unwrap_or(vec![]);
                        match event_type {
                            &SessionEventType::Login => {
                                dispatch.apply(LoginAction::Login(
                                    data.get("user_id")
                                        .map(|v| v.as_str().unwrap().to_owned())
                                        .unwrap(),
                                    data,
                                ));
                            }
                            &SessionEventType::LoginError => {
                                let context = contexts
                                    .get(event_request.acknowledgement.as_ref().unwrap())
                                    .unwrap();
                                let user_id = context
                                    .get("user_id")
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_string();
                                dispatch.apply(LoginAction::LoginError(data, user_id));
                            }
                            &SessionEventType::Logout => {
                                dispatch.apply(LoginAction::Logout(data));
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
                                            //TODO!!!!! following line should not be needed
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
                            &SessionEventType::Refreshed => match resource[0] {
                                Resource::Account => match &dispatch.get().data.storage_status {
                                    _ => {
                                        dbg!(&data);
                                        dispatch.apply(DataAction::ResourceFetched(
                                            Resource::Account,
                                            data.clone().into(),
                                            None,
                                        ));
                                    }
                                },
                                _ => {}
                            },
                            _ => {}
                        }
                    }
                    _ => {}
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
