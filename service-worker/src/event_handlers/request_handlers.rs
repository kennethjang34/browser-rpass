use browser_rpass::request::*;
use browser_rpass::response::*;
use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_NATIVE};
use browser_rpass::util::*;
use gloo_utils::format::JsValueSerdeExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::future::IntoFuture;
use std::str;
use std::str::from_utf8;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

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
        log!(
            "msg in native message callback: {:?}",
            String::from_utf8(msg.into()).unwrap()
        );
        match request {
            RequestEnum::Login(login_request) => {
                let login_response: LoginResponse =
                    serde_json::from_slice::<LoginResponse>(msg).unwrap();
                let session_storage = chrome.storage().session();
                wasm_bindgen_futures::spawn_local(async move {
                    session_storage
                        .set_string_item("passphrase".to_string(), login_request.passphrase.clone())
                        .await
                });
                let response = ResponseEnum::LoginResponse(login_response);
                return Ok(response);
            }
            RequestEnum::Get(get_request) => {
                let get_response: GetResponse = serde_json::from_slice::<GetResponse>(msg).unwrap();
                let response = ResponseEnum::GetResponse(get_response);
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
            if let Ok(passphrase) = chrome
                .storage()
                .session()
                .get_string_value("passphrase")
                .await
            {
                if let Some(passphrase) = passphrase {
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
                                    if let Some(ref acknowledgement) = request.get_acknowledgement()
                                    {
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
                                    if let Some(ref acknowledgement) = request.get_acknowledgement()
                                    {
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
                        RequestEnum::Search(search_request) => {
                            match search_request.resource.clone() {
                                Resource::Password => {
                                    let native_request_acknowledgement: String = {
                                        if let Some(ref acknowledgement) =
                                            request.get_acknowledgement()
                                        {
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
                                        <JsValue as JsValueSerdeExt>::from_serde(&search_request)
                                            .unwrap(),
                                    );
                                }
                                Resource::Username => {
                                    let native_request_acknowledgement: String = {
                                        if let Some(ref acknowledgement) =
                                            request.get_acknowledgement()
                                        {
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
                                        <JsValue as JsValueSerdeExt>::from_serde(&search_request)
                                            .unwrap(),
                                    );
                                }
                                _ => {
                                    let error_response = ErrorResponse {
                                        message: Some("resource not supported".to_owned()),
                                        acknowledgement: request.get_acknowledgement(),
                                        code: Some(ErrorCode::NotSupported),
                                    };
                                    extension_port.post_message(
                                        JsValue::from_serde(&error_response).unwrap(),
                                    );
                                }
                            }
                        }
                        RequestEnum::Init(init_request) => {
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
                                <JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap(),
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
                                create_request_callback(
                                    request.clone(),
                                    None,
                                    extension_port.clone(),
                                ),
                            );
                            log!("send login_request to native host: {:?}", login_request);
                            native_port.post_message(
                                <JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap(),
                            );
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
            } else {
                let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                    message: Some("passphrase is not set".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotAuthorized),
                });
                log!("error_response: {:?}", error_response);
                extension_port.post_message(JsValue::from_serde(&error_response).unwrap());
            }
        }
    });
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                wasm_bindgen_futures::spawn_local(async move {
                    let request: RequestEnum =
                        <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                    handle_request_from_popup(request, port, NATIVE_PORT.clone());
                });
            }
        });
        port.on_message().add_listener(cb.into_js_value());
    });
    return on_connect_with_popup_cb;
}
