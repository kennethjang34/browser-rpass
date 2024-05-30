use browser_rpass::{
    js_binding::extension_api::Port, request::SessionEventType, response::RequestEnum,
    types::Resource,
};
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use wasm_bindgen::{prelude::Closure, JsValue};
use yewdux::prelude::Dispatch;

use crate::store::{ContentScriptStore, DataAction, LoginAction};
pub fn create_message_listener(_port: &Port) -> Closure<dyn Fn(JsValue)> {
    Closure::<dyn Fn(JsValue)>::new(move |msg: JsValue| {
        match <JsValue as JsValueSerdeExt>::into_serde::<RequestEnum>(&msg) {
            Ok(request) => match request.clone() {
                RequestEnum::SessionEvent(session_event) => {
                    let dispatch = Dispatch::<ContentScriptStore>::new();
                    let event_request = session_event.clone();
                    let event_type = &event_request.event_type;
                    let event_detail = event_request.detail.clone().unwrap_or_default();
                    let resource = event_request.resource.unwrap_or(vec![]);
                    match event_type {
                        &SessionEventType::Init => {
                            dispatch.apply(DataAction::Init(event_detail));
                        }
                        &SessionEventType::Login => {
                            dispatch.apply(LoginAction::Login(event_detail.clone().into()));
                        }
                        &SessionEventType::Logout => {
                            dispatch.apply(LoginAction::Logout);
                        }
                        &SessionEventType::Delete => {
                            let resource = resource[0].clone();
                            match resource {
                                Resource::Account => {
                                    dispatch
                                        .apply(DataAction::ResourceDeleted(resource, event_detail));
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
                                        event_detail.clone().into(),
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
                                        resource,
                                        session_event,
                                    ));
                                }
                                _ => {}
                            }
                        }
                        &SessionEventType::Refreshed => match resource[0] {
                            Resource::Account => match &dispatch.get().data.storage_status {
                                _ => {
                                    dbg!(&event_detail);
                                    dispatch.apply(DataAction::ResourceFetched(
                                        Resource::Account,
                                        event_detail.clone().into(),
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
            Err(e) => {
                error!(
                    "error happend while parsing:{:?}. Error message: {:?}",
                    msg, e
                );
            }
        }
    })
}
