use browser_rpass::request::{GetRequest, Queryable, RequestEnum, SearchRequest};
use browser_rpass::response::{
    ErrorCode, ErrorResponse, GetResponse, ResponseEnum, SearchResponse,
};
use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_NATIVE};
use browser_rpass::util::*;
use gloo_utils::format::JsValueSerdeExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::future::IntoFuture;
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
            let serialized = serde_json::to_string(&parsed_response).unwrap();
            extension_port.post_message(JsValue::from_str(&serialized));
            Ok(())
        } else {
            Err(
                "error happened while parsing message from native host into search response"
                    .to_owned(),
            )
        }
    };
    Box::new(on_message_cb)
}

pub fn create_native_message_callback(
    request: RequestEnum,
    ctx: Option<HashMap<String, String>>,
) -> Box<impl FnOnce(&[u8], Port) -> Result<ResponseEnum, String> + Send + 'static> {
    let on_native_message_cb = move |msg: &[u8], native_port: Port| {
        return serde_json::from_slice::<ResponseEnum>(msg).map_err(|e| {
            format!(
                "error happened while parsing message from native host into search response. received message:{:?}, acknowledgement: {:?},error details: {:?}",
                msg,request.get_acknowledgement().clone(),
                e
            )
        });
    };
    Box::new(on_native_message_cb)
}

pub fn handle_request_from_popup(mut request: RequestEnum, port: Port, native_port: Port) {
    let request2 = request.clone();
    let port2 = port.clone();
    let native_port2 = native_port.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let mut request = request2;
        let port = port2;
        let native_port = native_port2;
        log!("request: {:?}", request);
        let passphrase_entry = chrome.storage().session().get("passphrase".into()).await;
        log!("passphrase_entry: {:?}", passphrase_entry);
        if let Ok(passphrase) = chrome.storage().session().get_value("passphrase").await {
            if let Some(passphrase) = passphrase.as_string() {
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
                                create_request_callback(request.clone(), None, port.clone()),
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
                                create_request_callback(request.clone(), None, port.clone()),
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
                            port.post_message(JsValue::from_serde(&error_response).unwrap());
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
                                create_request_callback(request.clone(), None, port.clone()),
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
                                create_request_callback(request.clone(), None, port.clone()),
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
                            port.post_message(JsValue::from_serde(&error_response).unwrap());
                        }
                    },
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
                            create_request_callback(request.clone(), None, port.clone()),
                        );
                        native_port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap(),
                        );
                    }
                }
            } else {
                let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                    message: Some("passphrase is not a string".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotSupported),
                });
                port.post_message(JsValue::from_serde(&error_response).unwrap());
            }
        } else {
            let error_response = ResponseEnum::ErrorResponse(ErrorResponse {
                message: Some("passphrase is not set".to_owned()),
                acknowledgement: request.get_acknowledgement(),
                code: Some(ErrorCode::NotAuthorized),
            });
            port.post_message(JsValue::from_serde(&error_response).unwrap());
        }
    });
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                wasm_bindgen_futures::spawn_local(async move {
                    let chrome_storage = chrome.storage().session();
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
