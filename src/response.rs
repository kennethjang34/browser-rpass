use core::fmt;
use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_repr::*;
use serde_variant::to_variant_name;
use std::fmt::Debug;
use wasm_bindgen::JsValue;

pub use crate::{request::RequestEnum, types::Resource};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
    pub resource: Resource,
    pub meta: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
    pub resource: Resource,
    pub meta: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
    pub resource: Resource,
    pub id: String,
    pub meta: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Vec<Value>,
    pub status: Status,
    pub resource: Resource,
    pub meta: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchResponse {
    pub acknowledgement: Option<String>,
    // #[serde(default)]
    // pub data: Value,
    pub status: Status,
    pub resource: Resource,
    pub data: Value,
    pub meta: Option<Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogoutResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteResponse {
    pub acknowledgement: Option<String>,
    #[serde(default)]
    pub data: Value,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Failure,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JSONData {
    pub payload: Value,
    #[serde(flatten)]
    pub meta: Option<Value>,
    pub resource: Resource,
}
pub struct JSONDataEntry {
    pub payload: Value,
    pub id: String,
    pub meta: Option<Value>,
    pub resource: Resource,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorResponse {
    pub acknowledgement: Option<String>,
    pub message: Option<String>,
    pub code: Option<ErrorCode>,
}
impl Into<JsValue> for ErrorResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for DeleteResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for InitResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for CreateResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for LogoutResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for GetResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for SearchResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for FetchResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl ResponseEnumTrait for ErrorResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn set_acknowledgement(&mut self, acknowledgement: Option<String>) {
        self.acknowledgement = acknowledgement;
    }
    fn get_data(&self) -> Value {
        let mut data = Map::new();
        if let Some(message) = &self.message {
            data.insert("message".to_owned(), serde_json::to_value(message).unwrap());
        }
        if let Some(code) = &self.code {
            data.insert("code".to_owned(), serde_json::to_value(code).unwrap());
        }
        data.into()
    }
    fn get_status(&self) -> Status {
        Status::Error
    }
}
macro_rules! response_enum_trait_impl {
    ($($t:ty)*) => ($(
        impl ResponseEnumTrait for $t {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn get_data(&self) -> Value{
        self.data.clone().into()
    }
    fn set_acknowledgement(&mut self, acknowledgement: Option<String>){
        self.acknowledgement = acknowledgement;
    }

    fn get_status(&self) -> Status{
        self.status.clone().into()
    }
        }

    )*)
}
response_enum_trait_impl!(GetResponse);
response_enum_trait_impl!(FetchResponse);
response_enum_trait_impl!(LoginResponse);
response_enum_trait_impl!(SearchResponse);
response_enum_trait_impl!(CreateResponse);
response_enum_trait_impl!(LogoutResponse);
// response_enum_trait_impl!(ErrorResponse);
response_enum_trait_impl!(InitResponse);
response_enum_trait_impl!(DeleteResponse);
response_enum_trait_impl!(EditResponse);

#[derive(Serialize, Deserialize, Debug, Clone)]
#[enum_dispatch(ResponseEnumTrait,Into<JsValue>)]
pub enum ResponseEnum {
    #[serde(rename = "get_response")]
    GetResponse(GetResponse),
    #[serde(rename = "search_response")]
    SearchResponse(SearchResponse),
    #[serde(rename = "fetch_response")]
    FetchResponse(FetchResponse),
    #[serde(rename = "login_response")]
    LoginResponse(LoginResponse),
    #[serde(rename = "error_response")]
    ErrorResponse(ErrorResponse),
    #[serde(rename = "init_response")]
    InitResponse(InitResponse),
    #[serde(rename = "logout_response")]
    LogoutResponse(LogoutResponse),
    #[serde(rename = "create_response")]
    CreateResponse(CreateResponse),
    #[serde(rename = "edit_response")]
    EditResponse(EditResponse),
    #[serde(rename = "delete_response")]
    DeleteResponse(DeleteResponse),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageEnum {
    Message(RequestEnum),
    Response(ResponseEnum),
}

#[enum_dispatch]
pub trait ResponseEnumTrait: Debug {
    fn get_acknowledgement(&self) -> Option<String>;
    fn get_data(&self) -> Value;
    fn set_acknowledgement(&mut self, acknowledgement: Option<String>);
    fn get_status(&self) -> Status;

    // fn get_response_type(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Data {
    String(String),
    JSON(Map<String, serde_json::Value>),
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResourceData {
    pub resource: Resource,
    pub data: Vec<Value>,
    pub meta: Option<Value>,
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u8)]
pub enum ErrorCode {
    NotAuthorized = 1,
    NotFound = 2,
    Unknown = 3,
    NotSupported = 4,
    Generic = 5,
    LoginFailed = 6,
    NativeAppConnectionError = 7,
}
impl fmt::Display for ResponseEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_variant_name(&self).unwrap())
    }
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_variant_name(&self).unwrap())
    }
}
impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_variant_name(&self).unwrap())
    }
}
