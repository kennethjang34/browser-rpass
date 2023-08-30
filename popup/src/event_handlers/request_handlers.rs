use std::collections::HashMap;

use crate::log;
use browser_rpass::response::ResponseEnumTrait;
use browser_rpass::store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP;
use browser_rpass::{request::RequestEnum, response::ResponseEnum, util::Port};
use wasm_bindgen::prelude::Closure;

//return value will be used as a callback for message that matches the acknowledgement(checked by the caller)
pub fn create_response_process_cb(
    request: RequestEnum,
    ctx: HashMap<String, String>,
) -> Box<impl FnOnce(ResponseEnum, Port) -> Result<(), String>> {
    let on_message_cb = move |response: ResponseEnum, extension_port: Port| -> Result<(), String> {
        log!("response: {:?}", response);
        Ok(())
    };
    Box::new(on_message_cb)
}
pub fn create_response_listener(port: Port) -> Closure<dyn Fn(String)> {
    Closure::<dyn Fn(String)>::new(move |msg: String| {
        match serde_json::from_str::<ResponseEnum>(&msg) {
            Ok(parsed_response) => {
                let acknowledgement = parsed_response.get_acknowledgement().unwrap();
                let request_cb = MESSAGE_ACKNOWLEDGEMENTS_POP_UP
                    .lock()
                    .unwrap()
                    .remove(&acknowledgement)
                    .unwrap();
                //this calls the callback that was registered in the popup (created by create_response_process_cb)
                request_cb(parsed_response, port.clone());
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
