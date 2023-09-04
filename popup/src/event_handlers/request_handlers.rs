use std::collections::HashMap;
use std::future::Future;

use crate::log;
use browser_rpass::response::ResponseEnumTrait;
use browser_rpass::store::{AsyncCallback, MESSAGE_ACKNOWLEDGEMENTS_POP_UP};
use browser_rpass::util::chrome;
use browser_rpass::StringOrCallback;
use browser_rpass::{request::RequestEnum, response::ResponseEnum, util::Port};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

//return value will be used as a callback for message that matches the acknowledgement(checked by the caller)
async fn handle_login_response(response: ResponseEnum, extension_port: Port) -> () {
    log!("handle_login_response");
    api_login_user(response, extension_port).await;
}
pub async fn api_login_user(response: ResponseEnum, extension_port: Port) -> Result<(), String> {
    log!(
        "api_login_user function called with response: {:?}",
        response
    );
    match response.clone() {
        ResponseEnum::LoginResponse(login_response) => {
            log!("login_response: {:?}", login_response);
            if let Ok(verified) = login_response.verified {
                chrome
                    .storage()
                    .session()
                    .set_string_item("verified".to_string(), verified.to_string())
                    .await;
            } else {
                return Err(format!(
                    "Error happened while performing login: {:?}",
                    login_response.verified
                )
                .to_owned());
            }
        }
        _ => {
            return Err("response type is not LoginResponse".to_owned());
        }
    }
    return Ok(());
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
            log!("login_response: {:?}", response.clone());
            Box::pin(handle_login_response(response, extension_port))
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
        // let val = <JsValue as JsValueSerdeExt>::from_serde(&msg).unwrap();
        match <JsValue as JsValueSerdeExt>::into_serde::<ResponseEnum>(&msg) {
            // match serde_json::from_str::<ResponseEnum>(&msg) {
            Ok(parsed_response) => {
                log!("parsed_response: {:?}", parsed_response);
                let acknowledgement = parsed_response.get_acknowledgement().unwrap();
                let request_cb = MESSAGE_ACKNOWLEDGEMENTS_POP_UP
                    .lock()
                    .unwrap()
                    .remove(&acknowledgement)
                    .unwrap();
                //this calls the callback that was registered in the popup (created by create_response_process_cb)
                let port2 = port.clone();
                spawn_local(async move {
                    log!("parsed_response: {:?}", parsed_response);
                    request_cb(parsed_response, port2.clone()).await;
                });
                // request_cb(parsed_response, port.clone()).await;
            }
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
