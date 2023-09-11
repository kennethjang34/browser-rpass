use enum_dispatch::enum_dispatch;
use gloo::storage::SessionStorage;
use gloo_utils::format::JsValueSerdeExt;
use serde_json::{Map, Value};
use std::{
    collections::HashMap,
    rc::{self, Rc},
};
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

use crate::util::create_request_acknowledgement;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "get")]
pub struct GetRequest {
    pub id: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "search")]
pub struct SearchRequest {
    pub query: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "login")]
pub struct LoginRequest {
    pub username: Option<String>,
    pub passphrase: String,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "logout")]
pub struct LogoutRequest {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "init")]
pub struct InitRequest {
    #[serde(flatten)]
    pub config: HashMap<String, String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub struct StorageUpdate {
    #[serde(flatten)]
    pub config: Option<HashMap<String, String>>,
    pub payload: Value,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename = "create")]
pub struct CreateRequest {
    pub username: String,
    // passphrase: Option<String>,
    pub path: String,
    pub value: Option<String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
impl RequestEnumTrait for CreateRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "create".to_owned()
    }
}

impl RequestEnumTrait for GetRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        self.id.clone()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "get".to_owned()
    }
}
impl RequestEnumTrait for SearchRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        self.query.clone()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "search".to_owned()
    }
}
impl RequestEnumTrait for StorageUpdate {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "storage_update".to_owned()
    }
}
impl RequestEnumTrait for InitRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        serde_json::to_string(&self.config).unwrap()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "init".to_owned()
    }
}
impl RequestEnumTrait for LoginRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "login".to_owned()
    }
}
impl RequestEnumTrait for LogoutRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_payload(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_type(&self) -> String {
        "logout".to_owned()
    }
}
impl Into<JsValue> for GetRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for SearchRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for InitRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for CreateRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for LoginRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for StorageUpdate {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for LogoutRequest {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[enum_dispatch(RequestEnumTrait)]
pub enum RequestEnum {
    #[serde(rename = "get")]
    Get(GetRequest),
    #[serde(rename = "search")]
    Search(SearchRequest),
    #[serde(rename = "login")]
    Login(LoginRequest),
    #[serde(rename = "logout")]
    Logout(LogoutRequest),
    #[serde(rename = "init")]
    Init(InitRequest),
    #[serde(rename = "create")]
    Create(CreateRequest),
    #[serde(rename = "storage_update")]
    StorageUpdate(StorageUpdate),
}
impl RequestEnum {
    pub fn create_get_request(
        id: String,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Get(GetRequest {
            id,
            resource,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
    pub fn create_create_request(
        username: String,
        path: String,
        value: Option<String>,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Create(CreateRequest {
            username,
            path,
            value,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
    pub fn create_search_request(
        query: String,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Search(SearchRequest {
            query,
            resource,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
    pub fn create_login_request(
        acknowledgement: Option<String>,
        username: Option<String>,
        passphrase: String,
    ) -> RequestEnum {
        RequestEnum::Login(LoginRequest {
            username,
            passphrase,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header: None,
        })
    }
    pub fn create_logout_request(
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Logout(LogoutRequest {
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
    pub fn create_init_request(
        config: HashMap<String, String>,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Init(InitRequest {
            config,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
    pub fn create_storage_update_request(
        config: Option<HashMap<String, String>>,
        state: Value,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::StorageUpdate(StorageUpdate {
            config,
            payload: state,
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            header,
        })
    }
}

#[enum_dispatch]
pub trait RequestEnumTrait {
    fn get_acknowledgement(&self) -> Option<String>;
    fn get_payload(&self) -> String;
    fn get_header(&self) -> Option<HashMap<String, String>>;
    fn set_header(&mut self, header: HashMap<String, String>);
    fn get_type(&self) -> String;
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Resource {
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "username")]
    Username,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Queryable {
    #[serde(rename = "password")]
    Password {
        path: Option<String>,
        username: String,
    },
    #[serde(rename = "username")]
    Username { path: String },
}
