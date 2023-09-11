use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;

use crate::log;
use crate::store::{LoginAction, PopupStore};
use browser_rpass::request::{self, InitRequest, LoginRequest, LogoutRequest};
use browser_rpass::response::{self, Data, MessageEnum, ResponseEnumTrait};
use browser_rpass::store::*;
use browser_rpass::util::chrome;
use browser_rpass::StringOrCallback;
use browser_rpass::{request::RequestEnum, response::ResponseEnum, util::Port};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::{dispatch, prelude::*};

//return value will be used as a callback for message that matches the acknowledgement(checked by the caller)
async fn handle_login_response(
    request: LoginRequest,
    response: ResponseEnum,
    extension_port: Port,
) {
    match response.clone() {
        ResponseEnum::LoginResponse(login_response) => {
            let dispatch = Dispatch::<PopupStore>::new();
            match login_response.status {
                response::Status::Success => {
                    dispatch.apply(LoginAction::LoginSucceeded);
                }
                response::Status::Failure => {
                    dispatch.apply(LoginAction::LoginFailed);
                }
                response::Status::Error => {
                    dispatch.apply(LoginAction::LoginError);
                }
                _ => {
                    log!(
                        "wrong response for request: {:?}. received response: format: {:?}",
                        request,
                        response
                    );
                }
            }
        }
        _ => {}
    }
}

async fn handle_init_response(request: InitRequest, response: ResponseEnum, extension_port: Port) {
    match response.clone() {
        ResponseEnum::InitResponse(init_response) => {
            let dispatch = Dispatch::<PopupStore>::new();
            if let Some(data) = init_response.data {
                match data {
                    Data::JSON(map_data) => {
                        log!("map_data from service worker: {:?}", map_data);
                        let new_store_state = PopupStore {
                            verified: map_data.get("verified").unwrap().as_bool().unwrap(),
                            ..Default::default()
                        };
                        dispatch.set(new_store_state);
                    }
                    _ => {}
                }
            } else {
            }
        }
        _ => {}
    }
}
async fn handle_logout_response(
    request: LogoutRequest,
    response: ResponseEnum,
    extension_port: Port,
) {
    match response.clone() {
        ResponseEnum::LogoutResponse(logout_response) => {
            let dispatch = Dispatch::<PopupStore>::new();
            match logout_response.status {
                response::Status::Success => {
                    dispatch.apply(LoginAction::LogoutSucceeded);
                }
                response::Status::Failure => {
                    dispatch.apply(LoginAction::LogoutFailed);
                }
                _ => {}
            }
        }
        _ => {
            log!(
                "wrong response for request: {:?}. received response: format: {:?}",
                request,
                response
            );
        }
    }
}
pub fn create_response_process_cb(
    request: RequestEnum,
    mut ctx: HashMap<String, StringOrCallback>,
) -> AsyncCallback {
    if let Some(callback) = ctx.remove("callback") {
        match callback {
            StringOrCallback::Callback(cb) => {
                cb();
            }
            _ => {}
        };
    }
    match request {
        RequestEnum::Login(login_request) => Box::new(move |response, extension_port| {
            Box::pin(handle_login_response(
                login_request,
                response,
                extension_port,
            ))
        }),
        RequestEnum::Init(init_request) => Box::new(move |response, extension_port| {
            Box::pin(handle_init_response(init_request, response, extension_port))
        }),
        RequestEnum::Logout(logout_request) => Box::new(move |response, extension_port| {
            Box::pin(handle_logout_response(
                logout_request,
                response,
                extension_port,
            ))
        }),
        _ => Box::new(|_, _| {
            Box::pin(async {
                log!("not implemented");
            })
        }),
    }
}
pub fn create_response_listener(port: Port) -> Closure<dyn Fn(JsValue)> {
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
                    }
                }
                MessageEnum::Request(request) => match request.clone() {
                    RequestEnum::StorageUpdate(storage_update_request) => {
                        let data = storage_update_request.payload;
                        let dispatch = Dispatch::<PopupStore>::new();
                        if let Ok(event) = serde_json::from_value(data) {
                            match event {
                                SessionEvent::LoginSucceeded => {
                                    dispatch.apply(LoginAction::LoginSucceeded);
                                }
                                SessionEvent::LogoutSucceeded => {
                                    dispatch.apply(LoginAction::LogoutSucceeded);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
            },
            Err(e) => {
                log!(
                    "error happend while parsing:{:?}. Error message: {:?}",
                    msg,
                    e
                );
            }
        }
    })
}
