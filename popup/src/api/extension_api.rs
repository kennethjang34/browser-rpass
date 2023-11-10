use crate::{store::LoginAction, Resource};

use browser_rpass::{request::RequestEnum, util::create_request_acknowledgement};
use gloo_utils::format::JsValueSerdeExt;
use serde_json::json;
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
pub fn login(user_id: String, passphrase: String) {
    let dispatch = Dispatch::<PopupStore>::new();
    dispatch.apply(LoginAction::LoginStarted(user_id.clone(), json!({})));
    let login_request = RequestEnum::create_login_request(
        Some(create_request_acknowledgement()),
        user_id,
        passphrase,
    );
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

pub fn create_account(path: String, username: String, password: String) -> String {
    let dispatch = Dispatch::<PopupStore>::new();
    dispatch.apply(DataAction::ResourceDeletionStarted(
        Resource::Account,
        json!({"path": path, "username": username, "password": password}),
    ));
    let acknowledgement = create_request_acknowledgement();
    let create_request = RequestEnum::create_create_request(
        username.clone(),
        path.clone(),
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
