use browser_rpass::request::{GetRequest, Queryable, RequestEnum, SearchRequest};
use browser_rpass::response::{GetResponse, ResponseEnum, SearchResponse};
use browser_rpass::store::{DATA_STORAGE, MESSAGE_ACKNOWLEDGEMENTS_NATIVE};
use browser_rpass::util::*;
use gloo_utils::format::JsValueSerdeExt;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

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

pub fn handle_request_from_popup(
    mut request: RequestEnum,
    port: Port,
    native_port: Port,
) -> Result<(), ()> {
    if let Some(passphrase) = DATA_STORAGE.lock().unwrap().get("passphrase") {
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
                    Ok(())
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
                    Ok(())
                }
                _ => Err(()),
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
                    Ok(())
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
                    Ok(())
                }
                _ => Err(()),
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
                native_port
                    .post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());
                Ok(())
            }
        }
    } else {
        return Err(());
    }
}
pub fn create_request_listener() -> Closure<dyn Fn(Port)> {
    let on_connect_with_popup_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        // let native_port = native_port.clone();
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                let request: RequestEnum = <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                handle_request_from_popup(request, port, NATIVE_PORT.clone()).unwrap();
            }
        });
        port.on_message().add_listener(cb.into_js_value());
    });
    return on_connect_with_popup_cb;
    // on_connect_with_popup_cb(extension_port, native_port)
}
