use core::fmt;
use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_repr::*;
use serde_variant::to_variant_name;
use std::{collections::HashMap, fmt::Debug, path::PathBuf};
use wasm_bindgen::JsValue;

use crate::request::DataFieldType;
pub use crate::{request::RequestEnum, types::Resource};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
    pub resource: Resource,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResponse {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub store_id: String,
    pub status: Status,
    pub resource: Resource,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateStoreResponse {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub store_path: PathBuf,
    pub store_id: String,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteStoreResponse {
    pub acknowledgement: Option<String>,
    pub store_id: String,
    pub status: Status,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
}
impl CreateStoreResponse {
    pub fn new(
        store_id: String,
        store_path: PathBuf,
        status: Status,
        acknowledgement: Option<String>,
        detail: Option<HashMap<DataFieldType, Value>>,
    ) -> Self {
        Self {
            store_id,
            store_path,
            status,
            detail: detail.unwrap_or_default(),
            acknowledgement,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditResponse {
    pub instance_id: String,
    pub store_id: String,
    pub status: Status,
    pub resource: Resource,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub acknowledgement: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateLog {
    pub field: DataFieldType,
    pub old: Value,
    pub new: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponse {
    pub store_id: String,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
    pub resource: Resource,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchResponse {
    pub acknowledgement: Option<String>,
    pub status: Status,
    pub store_id: String,
    pub resource: Resource,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub store_id: String,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitResponse {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogoutResponse {
    pub store_id: Option<String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteResponse {
    pub deleted_resource_id: String,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericError {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub detail: HashMap<DataFieldType, Value>,
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
    pub detail: Option<Value>,
    pub resource: Resource,
}
pub struct JSONDataEntry {
    pub payload: Value,
    pub id: String,
    pub detail: Option<Value>,
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
impl Into<JsValue> for GenericError {
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
impl Into<JsValue> for DeleteStoreResponse {
    fn into(self) -> JsValue {
        <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
    }
}
impl Into<JsValue> for CreateStoreResponse {
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
    fn get_detail(&self) -> HashMap<DataFieldType, Value> {
        let mut detail = HashMap::new();
        if let Some(message) = &self.message {
            detail.insert(
                DataFieldType::ErrorMessage,
                serde_json::to_value(message).unwrap(),
            );
        }
        if let Some(code) = &self.code {
            detail.insert(
                DataFieldType::ErrorCode,
                serde_json::to_value(code).unwrap(),
            );
        }
        detail.into()
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
    fn get_detail(&self) -> HashMap<DataFieldType, Value>{
        self.detail.clone()
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
response_enum_trait_impl!(GenericError);
response_enum_trait_impl!(SearchResponse);
response_enum_trait_impl!(CreateStoreResponse);
response_enum_trait_impl!(CreateResponse);
response_enum_trait_impl!(LogoutResponse);
response_enum_trait_impl!(InitResponse);
response_enum_trait_impl!(DeleteResponse);
response_enum_trait_impl!(EditResponse);
response_enum_trait_impl!(DeleteStoreResponse);

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
    #[serde(rename = "create_store_response")]
    CreateStoreResponse(CreateStoreResponse),
    #[serde(rename = "edit_response")]
    EditResponse(EditResponse),
    #[serde(rename = "delete_response")]
    DeleteResponse(DeleteResponse),
    #[serde(rename = "delete_store_response")]
    DeleteStoreResponse(DeleteStoreResponse),
    #[serde(rename = "generic_error")]
    GenericError(GenericError),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageEnum {
    Message(RequestEnum),
}

#[enum_dispatch]
pub trait ResponseEnumTrait: Debug {
    fn get_acknowledgement(&self) -> Option<String>;
    fn get_detail(&self) -> HashMap<DataFieldType, Value>;
    fn set_acknowledgement(&mut self, acknowledgement: Option<String>);
    fn get_status(&self) -> Status;
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
    pub detail: Option<Value>,
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
