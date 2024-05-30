use browser_rpass::{
    js_binding::extension_api::*,
    request::{NotificationTarget, SessionEvent, SessionEventType},
    response::RequestEnum,
};
use gloo_utils::format::JsValueSerdeExt;
#[allow(unused_imports)]
use log::debug;
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
    let mut store_listeners = LISTENER_PORT.lock().unwrap();
    let mut target_ports: Vec<&Port> = Vec::new();
    let ports_map = EXTENSION_PORT.lock().unwrap();
    match session_event.notification_target {
        NotificationTarget::All => {
            for port in ports_map.values() {
                target_ports.push(port);
            }
        }
        NotificationTarget::Store { ref store_id } => {
            let listeners = store_listeners.get_mut(store_id);
            if let Some(listeners) = listeners {
                for port in ports_map.values() {
                    if listeners.contains(&port.name()) {
                        target_ports.push(port);
                    }
                }
            }
        }
        NotificationTarget::Port { ref port_id } => {
            if let Some(port) = ports_map.get(port_id) {
                target_ports.push(port);
            }
        }
        NotificationTarget::Ports { ref port_ids } => {
            for port_name in port_ids {
                if let Some(port) = ports_map.get(port_name) {
                    target_ports.push(port);
                }
            }
        }
        NotificationTarget::StoreAndPorts {
            ref store_id,
            ref port_ids,
        } => {
            for port_name in port_ids {
                if let Some(port) = ports_map.get(port_name) {
                    target_ports.push(port);
                }
            }
            let listeners = store_listeners.get_mut(store_id);
            if let Some(listeners) = listeners {
                for port in ports_map.values() {
                    if listeners.contains(&port.name()) {
                        target_ports.push(port);
                    }
                }
            }
        }
        NotificationTarget::None => {}
    }
    for port in target_ports {
        let request =
            RequestEnum::create_session_event_request(None, session_event.clone(), None, None);
        port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&request).unwrap());
    }
    if let Some(ports_to_disconnect) = ports_to_disconnect {
        for (store_id_index, ports) in ports_to_disconnect {
            let listeners = store_listeners.get_mut(&store_id_index);
            if let Some(listeners) = listeners {
                for port_name in ports {
                    listeners.remove(&port_name);
                }
            }
        }
    }
}
pub fn whisper_session_event(session_event: SessionEvent, port: &Port) {
    let msg = RequestEnum::create_session_event_request(None, session_event, None, None);
    port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&msg).unwrap());
}
