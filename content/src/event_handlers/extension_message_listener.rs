use browser_rpass::{
    request::SessionEventType,
    response::{MessageEnum, RequestEnum, ResponseEnumTrait},
    store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP,
    types::Resource,
    Port,
};
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use serde_json::json;
use wasm_bindgen::{prelude::Closure, JsValue};
use wasm_bindgen_futures::spawn_local;
use yewdux::prelude::Dispatch;

use crate::store::{ContentScriptStore, DataAction, LoginAction};
pub fn create_message_listener(port: &Port) -> Closure<dyn Fn(JsValue)> {
    let port = port.clone();
    Closure::<dyn Fn(JsValue)>::new(move |msg: JsValue| {
        info!("msg received in ContentScript: {:?}", msg);
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
                    }
                }
                MessageEnum::Message(request) => match request.clone() {
                    RequestEnum::SessionEventRequest(request) => {
                        dbg!(&request);
                        let dispatch = Dispatch::<ContentScriptStore>::new();
                        let event_request = request.session_event.clone();
                        let event_type = &event_request.event_type;
                        let data = event_request.data.clone().unwrap_or(json!({}));

                        let resource = event_request.resource.unwrap_or(vec![]);
                        match event_type {
                            &SessionEventType::Login => {
                                dispatch.apply(LoginAction::Login);
                            }
                            &SessionEventType::Logout => {
                                dispatch.apply(LoginAction::Logout);
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
                info!(
                    "error happend while parsing:{:?}. Error message: {:?}",
                    msg, e
                );
            }
        }
    })
}
