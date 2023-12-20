use crate::{store::LoginAction, Resource};

use browser_rpass::{
    request::RequestEnum, store::MESSAGE_CONTEXT_POPUP, util::create_request_acknowledgement,
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
    dispatch.apply(LoginAction::LoginStarted(store_id.clone(), json!({})));
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
    dispatch.apply(LoginAction::LogoutStarted(json!({})));
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
    dispatch.apply(DataAction::ResourceCreationStarted(
        Resource::Account,
        json!({"username": username, "password": password, "domain": domain, "note": note}),
    ));
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
    let mut payload = json!({"id": id});
    if let Some(username) = username.as_ref() {
        payload
            .as_object_mut()
            .unwrap()
            .insert("username".into(), Value::String(username.clone()));
    }
    if let Some(note) = note.as_ref() {
        payload
            .as_object_mut()
            .unwrap()
            .insert("note".into(), Value::String(note.clone()));
    }
    if let Some(domain) = domain.as_ref() {
        payload
            .as_object_mut()
            .unwrap()
            .insert("domain".into(), Value::String(domain.clone()));
    }
    if let Some(password) = password.as_ref() {
        payload
            .as_object_mut()
            .unwrap()
            .insert("password".into(), Value::String(password.clone()));
    }
    dispatch.apply(DataAction::ResourceEditionStarted(
        Resource::Account,
        payload,
    ));
    let acknowledgement = create_request_acknowledgement();
    let edit_request = RequestEnum::create_edit_request(
        id,
        Resource::Account,
        domain.clone(),
        json!({"username": username, "password": password,"domain": domain, "note": note}),
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
    dispatch.apply(DataAction::ResourceDeletionStarted(
        resource.clone(),
        json!({"id": id}),
    ));
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
