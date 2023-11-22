use browser_rpass::{
    dbg,
    request::{SessionEvent, SessionEventType},
    response::{LoginResponse, MessageEnum, RequestEnum},
    Port,
};
use gloo_utils::format::JsValueSerdeExt;
use log::info;
use wasm_bindgen::JsValue;

use crate::store::{EXTENSION_PORT, LISTENER_PORT};

pub fn broadcast_session_event(session_event: SessionEvent) {
    info!("broadcast_session_event: {:?}", session_event);
    if let Some(ref resources) = session_event.resource {
        for resource in resources {
            let locked = LISTENER_PORT.lock().unwrap();
            let listeners = locked.get(resource);
            dbg!(&listeners);
            if let Some(listeners) = listeners {
                for port in EXTENSION_PORT.lock().unwrap().values() {
                    if listeners.contains(&port.name()) {
                        let request =
                            MessageEnum::Message(RequestEnum::create_session_event_request(
                                None,
                                session_event.clone(),
                                None,
                            ));
                        port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&request).unwrap(),
                        );
                    }
                }
            }
        }
    } else {
        for port in EXTENSION_PORT.lock().unwrap().values() {
            if let Some(ref _meta) = session_event.meta {}
            let request = MessageEnum::Message(RequestEnum::create_session_event_request(
                None,
                session_event.clone(),
                None,
            ));
            port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&request).unwrap());
        }
    }
}
pub fn whisper_session_event(session_event: SessionEvent, port: &Port) {
    let msg = MessageEnum::Message(RequestEnum::create_session_event_request(
        None,
        session_event,
        None,
    ));
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&msg).unwrap());
}
