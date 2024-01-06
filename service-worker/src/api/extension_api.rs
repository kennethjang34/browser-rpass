use browser_rpass::{
    js_binding::extension_api::*,
    request::{SessionEvent, SessionEventType},
    response::{MessageEnum, RequestEnum},
};
use gloo_utils::format::JsValueSerdeExt;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::JsValue;

use crate::store::{EXTENSION_PORT, LISTENER_PORT};

pub fn broadcast_session_event(
    session_event: SessionEvent,
    ports_to_disconnect: Option<HashMap<String, HashSet<String>>>,
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
    if session_event.store_id_index.is_none() {
        for port in EXTENSION_PORT.lock().unwrap().values() {
            let request = MessageEnum::Message(RequestEnum::create_session_event_request(
                None,
                session_event.clone(),
                None,
                None,
            ));
            port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&request).unwrap());
        }
        return;
    } else {
        let listeners = locked.get_mut(session_event.store_id_index.as_ref().unwrap());
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
        }
    }
    if let Some(ports_to_disconnect) = ports_to_disconnect {
        for (store_id_index, ports) in ports_to_disconnect {
            let listeners = locked.get_mut(&store_id_index);
            if let Some(listeners) = listeners {
                for port_name in ports {
                    listeners.remove(&port_name);
                }
            }
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
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&msg).unwrap());
}
