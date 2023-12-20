use std::collections::HashMap;

use crate::{store::LoginAction, Resource};

use browser_rpass::{
    request::{DataFieldType, RequestEnum},
    store::MESSAGE_CONTEXT_POPUP,
    util::create_request_acknowledgement,
};
use gloo_utils::format::JsValueSerdeExt;
use serde_json::{json, Value};
use wasm_bindgen::JsValue;
use yewdux;
use yewdux::prelude::Dispatch;

use crate::store::{DataAction, PopupStore, EXTENSION_PORT};

//send fetch request to native app for the given path, return acknowledgement of the request
pub fn fetch_accounts(path: Option<String>) -> String {
    let dispatch = Dispatch::<PopupStore>::new();
    let acknowledgement = create_request_acknowledgement();
    let fetch_request = RequestEnum::create_fetch_request(
        path,
        Resource::Account,
        Some(acknowledgement.clone()),
        None,
    );
    dispatch.apply(DataAction::ResourceFetchStarted(Resource::Account));
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&fetch_request).unwrap());
    return acknowledgement;
}
pub fn login(store_id: String) {
    let dispatch = Dispatch::<PopupStore>::new();
    dispatch.apply(LoginAction::LoginStarted(store_id.clone(), HashMap::new()));
    let acknowledgement = create_request_acknowledgement();
    let login_request =
        RequestEnum::create_login_request(Some(acknowledgement.clone()), store_id.clone());
    MESSAGE_CONTEXT_POPUP
        .lock()
        .unwrap()
        .insert(acknowledgement, json!({"store_id":store_id.clone()}));
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&login_request).unwrap());
}
pub fn logout() {
    let dispatch = Dispatch::<PopupStore>::new();
    dispatch.apply(LoginAction::LogoutStarted(HashMap::new()));
    let logout_request = RequestEnum::create_logout_request(None, None);
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&logout_request).unwrap());
}

pub fn create_account(
    domain: Option<String>,
    username: Option<String>,
    password: Option<String>,
    note: Option<String>,
) -> String {
    let dispatch = Dispatch::<PopupStore>::new();
    let mut data = HashMap::new();
    data.insert(
        DataFieldType::Username,
        Value::String(username.clone().unwrap()),
    );
    data.insert(
        DataFieldType::Password,
        Value::String(password.clone().unwrap()),
    );
    data.insert(
        DataFieldType::Domain,
        Value::String(domain.clone().unwrap()),
    );
    data.insert(DataFieldType::Note, Value::String(note.clone().unwrap()));
    dispatch.apply(DataAction::ResourceCreationStarted(Resource::Account, data));
    let acknowledgement = create_request_acknowledgement();
    let create_request = RequestEnum::create_create_request(
        username.clone(),
        domain.clone(),
        note.clone(),
        None,
        Resource::Account,
        password.clone().into(),
        Some(acknowledgement.clone()),
        None,
    );
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&create_request).unwrap());
    return acknowledgement;
}

pub fn edit_account(
    id: String,
    domain: Option<String>,
    username: Option<String>,
    password: Option<String>,
    note: Option<String>,
) -> String {
    let dispatch = Dispatch::<PopupStore>::new();
    let mut payload = HashMap::new();
    payload.insert(DataFieldType::ResourceID, Value::String(id.clone()));
    // json!({"id": id});
    if let Some(username) = username.as_ref() {
        payload
            // .as_object_mut()
            // .unwrap()
            .insert(DataFieldType::Username, Value::String(username.clone()));
    }
    if let Some(note) = note.as_ref() {
        payload
            // .as_object_mut()
            // .unwrap()
            // .insert("note".into(), Value::String(note.clone()));
            .insert(DataFieldType::Note, Value::String(note.clone()));
    }
    if let Some(domain) = domain.as_ref() {
        payload
            // .as_object_mut()
            // .unwrap()
            .insert(DataFieldType::Domain, Value::String(domain.clone()));
    }
    if let Some(password) = password.as_ref() {
        payload
            // .as_object_mut()
            // .unwrap()
            .insert(DataFieldType::Password, Value::String(password.clone()));
    }
    dispatch.apply(DataAction::ResourceEditionStarted(
        Resource::Account,
        payload,
    ));
    let acknowledgement = create_request_acknowledgement();
    let mut data = HashMap::new();
    data.insert(
        DataFieldType::Username,
        serde_json::to_value(username.clone()).unwrap_or_default(),
    );
    data.insert(
        DataFieldType::Password,
        serde_json::to_value(password.clone()).unwrap_or_default(),
    );
    data.insert(
        DataFieldType::Domain,
        serde_json::to_value(domain.clone()).unwrap_or_default(),
    );
    data.insert(
        DataFieldType::Note,
        serde_json::to_value(note.clone()).unwrap_or_default(),
    );
    data.insert(
        DataFieldType::ResourceID,
        serde_json::to_value(id.clone()).unwrap_or_default(),
    );
    let edit_request = RequestEnum::create_edit_request(
        id,
        Resource::Account,
        domain.clone(),
        data,
        // json!({"username": username, "password": password,"domain": domain, "note": note}),
        Some(acknowledgement.clone()),
        None,
    );
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&edit_request).unwrap());
    return acknowledgement;
}

pub fn delete_resource(id: String, resource: Resource) -> String {
    let dispatch = Dispatch::<PopupStore>::new();
    let mut data = HashMap::new();
    data.insert(DataFieldType::ResourceID, Value::String(id.clone()));
    dispatch.apply(DataAction::ResourceDeletionStarted(resource.clone(), data));
    let acknowledgement = create_request_acknowledgement();
    let delete_request = RequestEnum::create_delete_request(
        id.clone(),
        resource,
        Some(acknowledgement.clone()),
        None,
    );
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&delete_request).unwrap());
    return acknowledgement;
}
