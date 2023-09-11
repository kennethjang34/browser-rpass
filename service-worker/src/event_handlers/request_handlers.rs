use crate::store::set_passphrase;
use crate::store::set_passphrase_async;
use crate::store::set_verified_status;
use crate::store::set_verified_status_async;
use crate::store::SessionAction;
use crate::store::SessionStore;
use crate::store::EXTENSION_PORT;
use browser_rpass::request::*;
use browser_rpass::response::*;
use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_NATIVE};
use browser_rpass::util::*;
use gloo_utils::format::JsValueSerdeExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::json;
use serde_json::Map;
use serde_json::Value;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::error::Error;
use std::future::IntoFuture;
use std::ops::DerefMut;
use std::str;
use std::str::from_utf8;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use yewdux::dispatch;
use yewdux::prelude::Dispatch;

use browser_rpass::log;
use browser_rpass::request::*;
use browser_rpass::util::*;

use crate::store::NATIVE_PORT;
pub fn create_request_callback(
    request: RequestEnum,
    ctx: Option<HashMap<String, String>>,
    extension_port: Port,
) -> Box<impl FnOnce(&[u8], Port) -> Result<(), String>> {
    let native_messaging_cb = create_native_message_callback(request.clone(), ctx);
    let on_message_cb = move |msg: &[u8], native_port: Port| {
        let parsed: Result<ResponseEnum, String> = native_messaging_cb(msg.clone(), native_port);
        if let Ok(parsed_response) = parsed {
            let s = str::from_utf8(msg).unwrap();
            extension_port
                .post_message(<JsValue as JsValueSerdeExt>::from_serde(&parsed_response).unwrap());
            Ok(())
        } else {
            log!(
                "error happened while parsing message from native host into search response. received message:{:?}, acknowledgement: {:?},error details: {:?}",
                msg,request.get_acknowledgement().clone(),
                parsed
            );
            let err = Err(
                "error happened while parsing message from native host into search response"
                    .to_owned(),
            );
            extension_port.post_message(JsValue::from_str(&format!(
                "error happened while parsing message from native host into search response. received message:{:?}, acknowledgement: {:?},error details: {:?}",
                msg,request.get_acknowledgement().clone(),
                err
            )));

            err
        }
    };
    Box::new(on_message_cb)
}

pub fn create_native_message_callback(
    request: RequestEnum,
    ctx: Option<HashMap<String, String>>,
) -> Box<impl FnOnce(&[u8], Port) -> Result<ResponseEnum, String> + Send + 'static> {
    let on_native_message_cb = move |msg: &[u8], native_port: Port| {
        match request {
            RequestEnum::Login(login_request) => {
                let login_response: LoginResponse =
                    serde_json::from_slice::<LoginResponse>(msg).unwrap();
                let session_storage = chrome.storage().session();
                let session_store_dispatch = Dispatch::<SessionStore>::new();
                let login_response2 = login_response.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    log!(
                        "all values in service worker session storage: {:?}",
                        session_storage.get_all().await
                    );
                    let login_response = login_response2;
                    match login_response.status {
                        Status::Success => {
                            set_verified_status(true, session_store_dispatch.clone());
                            set_passphrase(
                                Some(login_request.passphrase),
                                session_store_dispatch.clone(),
                            )
                        }
                        Status::Failure => {
                            set_verified_status(false, session_store_dispatch.clone());
                            set_passphrase(None, session_store_dispatch.clone())
                        }
                        _ => {}
                    };
                });
                let response = ResponseEnum::LoginResponse(login_response);
                return Ok(response);
            }
            RequestEnum::Get(get_request) => {
                let get_response: GetResponse = serde_json::from_slice::<GetResponse>(msg).unwrap();
                let response = ResponseEnum::GetResponse(get_response);
                return Ok(response);
            }
            RequestEnum::Create(create_request) => {
                let create_response: CreateResponse =
                    serde_json::from_slice::<CreateResponse>(msg).unwrap();
                let response = ResponseEnum::CreateResponse(create_response);
                return Ok(response);
            }
            RequestEnum::Search(search_request) => {
                let search_response: SearchResponse =
                    serde_json::from_slice::<SearchResponse>(msg).unwrap();
                let response = ResponseEnum::SearchResponse(search_response);
                return Ok(response);
            }
            RequestEnum::Init(init_request) => {
                let init_response: InitResponse =
                    serde_json::from_slice::<InitResponse>(msg).unwrap();
                let response = ResponseEnum::InitResponse(init_response);
                return Ok(response);
            }
            _ => {
                let error_response = ErrorResponse {
                    message: Some("resource not supported".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotSupported),
                };
                return Ok(ResponseEnum::ErrorResponse(error_response));
            }
        };
        // return serde_json::from_slice::<ResponseEnum>(msg).map_err(|e| {
        //     format!(
        //         "error happened while parsing message from native host into search response. received message:{:?}, acknowledgement: {:?},error details: {:?}",
        //         msg,request.get_acknowledgement().clone(),
        //         e
        //     )
        // });
    };
    Box::new(on_native_message_cb)
}

pub fn handle_request_from_popup(
    mut request: RequestEnum,
    extension_port: Port,
    native_port: Port,
) {
    log!("request: {:?}", request);
    let dispatch = Dispatch::<SessionStore>::new();
    let request2 = request.clone();
    let extension_port2 = extension_port.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let mut request = request2;
        let extension_port = extension_port2;
        let native_port = NATIVE_PORT.clone();
        if request.get_type() == "login" {
            match request.clone() {
                RequestEnum::Login(login_request) => {
                    let native_request_acknowledgement: String = {
                        if let Some(ref acknowledgement) = request.get_acknowledgement() {
                            acknowledgement.clone()
                        } else {
                            create_request_acknowledgement()
                        }
                    };
                    MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                        native_request_acknowledgement.clone(),
                        create_request_callback(request.clone(), None, extension_port.clone()),
                    );
                    native_port.post_message(
                        <JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap(),
                    );
                }
                _ => {
                    let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                        message: Some("login request malformed".to_owned()),
                        acknowledgement: request.get_acknowledgement(),
                        code: Some(ErrorCode::NotSupported),
                    });
                    log!("error_response: {:?}", error_response);
                    extension_port.post_message(JsValue::from_serde(&error_response).unwrap());
                }
            }
        } else {
            log!("store: {:?}", dispatch.get());
            log!("request: {:?}", request);
            if let Some(passphrase) = dispatch.get().passphrase.clone() {
                let header_map = {
                    let mut map = HashMap::new();
                    map.insert("passphrase".to_owned(), passphrase.clone());
                    map
                };
                request.set_header(header_map);
                match request.clone() {
                    RequestEnum::Get(get_request) => match get_request.resource.clone() {
                        Resource::Password => {
                            let native_request_acknowledgement: String = {
                                if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                    acknowledgement.clone()
                                } else {
                                    create_request_acknowledgement()
                                }
                            };
                            MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                                native_request_acknowledgement.clone(),
                                create_request_callback(
                                    request.clone(),
                                    None,
                                    extension_port.clone(),
                                ),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&get_request).unwrap(),
                            );
                        }
                        Resource::Username => {
                            let native_request_acknowledgement: String = {
                                if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                    acknowledgement.clone()
                                } else {
                                    create_request_acknowledgement()
                                }
                            };
                            MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                                native_request_acknowledgement.clone(),
                                create_request_callback(
                                    request.clone(),
                                    None,
                                    extension_port.clone(),
                                ),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&get_request).unwrap(),
                            );
                        }
                        _ => {
                            let error_response = ErrorResponse {
                                message: Some("resource not supported".to_owned()),
                                acknowledgement: request.get_acknowledgement(),
                                code: Some(ErrorCode::NotSupported),
                            };
                            extension_port
                                .post_message(JsValue::from_serde(&error_response).unwrap());
                        }
                    },
                    RequestEnum::Search(search_request) => match search_request.resource.clone() {
                        Resource::Password => {
                            let native_request_acknowledgement: String = {
                                if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                    acknowledgement.clone()
                                } else {
                                    create_request_acknowledgement()
                                }
                            };
                            MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                                native_request_acknowledgement.clone(),
                                create_request_callback(
                                    request.clone(),
                                    None,
                                    extension_port.clone(),
                                ),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&search_request).unwrap(),
                            );
                        }
                        Resource::Username => {
                            let native_request_acknowledgement: String = {
                                if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                    acknowledgement.clone()
                                } else {
                                    create_request_acknowledgement()
                                }
                            };
                            MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                                native_request_acknowledgement.clone(),
                                create_request_callback(
                                    request.clone(),
                                    None,
                                    extension_port.clone(),
                                ),
                            );
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&search_request).unwrap(),
                            );
                        }
                        _ => {
                            let error_response = ErrorResponse {
                                message: Some("resource not supported".to_owned()),
                                acknowledgement: request.get_acknowledgement(),
                                code: Some(ErrorCode::NotSupported),
                            };
                            extension_port
                                .post_message(JsValue::from_serde(&error_response).unwrap());
                        }
                    },
                    RequestEnum::Init(init_request) => {
                        let mut map = Map::new();
                        map.insert("verified".to_owned(), Value::Bool(dispatch.get().verified));
                        map.insert(
                            "acknowledgement".to_owned(),
                            Value::String(
                                init_request
                                    .acknowledgement
                                    .clone()
                                    .unwrap_or("".to_string()),
                            ),
                        );
                        let init_response = ResponseEnum::InitResponse(InitResponse {
                            acknowledgement: init_request.acknowledgement.clone(),
                            data: Some(Data::JSON(map)),
                        });
                        log!("init_response: {:?}", init_response);
                        extension_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&init_response).unwrap(),
                        );
                    }
                    RequestEnum::Login(login_request) => {
                        let native_request_acknowledgement: String = {
                            if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                acknowledgement.clone()
                            } else {
                                create_request_acknowledgement()
                            }
                        };
                        MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                            native_request_acknowledgement.clone(),
                            create_request_callback(request.clone(), None, extension_port.clone()),
                        );
                        native_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap(),
                        );
                    }
                    RequestEnum::Logout(logout_request) => {
                        let dispatch = Dispatch::<SessionStore>::new();
                        dispatch.apply(SessionAction::Logout);
                        extension_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(
                                &ResponseEnum::LogoutResponse(LogoutResponse {
                                    acknowledgement: request.get_acknowledgement(),
                                    status: Status::Success,
                                }),
                            )
                            .unwrap(),
                        );
                        log!("after logout apply_callback called");
                    }
                    RequestEnum::Create(create_request) => {
                        let native_request_acknowledgement: String = {
                            if let Some(ref acknowledgement) = request.get_acknowledgement() {
                                acknowledgement.clone()
                            } else {
                                create_request_acknowledgement()
                            }
                        };
                        MESSAGE_ACKNOWLEDGEMENTS_NATIVE.lock().unwrap().insert(
                            native_request_acknowledgement.clone(),
                            create_request_callback(request.clone(), None, extension_port.clone()),
                        );
                        native_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&create_request).unwrap(),
                        );
                    }
                    _ => {
                        let error_response = ErrorResponse {
                            message: Some("resource not supported".to_owned()),
                            acknowledgement: request.get_acknowledgement(),
                            code: Some(ErrorCode::NotSupported),
                        };
                        extension_port.post_message(JsValue::from_serde(&error_response).unwrap());
                    }
                }
            } else {
                let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                    message: Some("passphrase is not a string".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotSupported),
                });
                log!("error_response: {:?}", error_response);
                extension_port.post_message(JsValue::from_serde(&error_response).unwrap());
            }
        }
    });
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        if EXTENSION_PORT.lock().unwrap().is_none() {
            port.on_disconnect().add_listener({
                let cloned_port = port.clone();
                Closure::<dyn Fn(Port)>::new(move |port: Port| {
                    *EXTENSION_PORT.lock().unwrap() = None;
                })
                .into_js_value()
            });
        }
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                wasm_bindgen_futures::spawn_local(async move {
                    let request: RequestEnum =
                        <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                    log!("msg: {:?}", msg);
                    handle_request_from_popup(request, port, NATIVE_PORT.clone());
                });
            }
        });
        port.on_message().add_listener(cb.into_js_value());
    });
    return on_connect_with_popup_cb;
}
