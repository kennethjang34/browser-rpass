use std::collections::HashSet;

use browser_rpass::{
    js_binding::extension_api::*,
    request::{SessionEvent, SessionEventType},
    response::{MessageEnum, RequestEnum},
};
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use wasm_bindgen::JsValue;

use crate::store::{EXTENSION_PORT, LISTENER_PORT};

pub fn broadcast_session_event(
    session_event: SessionEvent,
    ports_to_disconnect: Option<HashSet<String>>,
) {
    let event_type = session_event.event_type.clone();
    match event_type {
        SessionEventType::Create
        | SessionEventType::Update
        | SessionEventType::Delete
        | SessionEventType::Refreshed => {}
        _ => {}
    }
    let mut locked = LISTENER_PORT.lock().unwrap();
    let listeners = locked.get_mut(session_event.store_id.as_ref().unwrap());
    if let Some(listeners) = listeners {
        for port in EXTENSION_PORT.lock().unwrap().values() {
            if listeners.contains(&port.name()) {
                let request = MessageEnum::Message(RequestEnum::create_session_event_request(
                    None,
                    session_event.clone(),
                    None,
                    None,
                ));
                port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&request).unwrap());
            }
        }
        if let Some(ports_to_disconnect) = ports_to_disconnect.clone() {
            listeners.retain(|port_name| !ports_to_disconnect.contains(port_name));
        }
    }
}
pub fn whisper_session_event(session_event: SessionEvent, port: &Port) {
    let msg = MessageEnum::Message(RequestEnum::create_session_event_request(
        None,
        session_event,
        None,
        None,
    ));
    debug!("whisper_session_event: {:?}", msg);
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&msg).unwrap());
}
