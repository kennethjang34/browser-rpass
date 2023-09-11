use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use serde_repr::*;
use std::fmt::Debug;
use wasm_bindgen::JsValue;

use crate::request::RequestEnum;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub acknowledgement: Option<String>,
    pub data: Option<Data>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateResponse {
    pub acknowledgement: Option<String>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponse {
    pub acknowledgement: Option<String>,
    pub data: Option<Vec<Data>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginResponse {
    pub acknowledgement: Option<String>,
    pub status: Status,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitResponse {
    pub acknowledgement: Option<String>,
    pub data: Option<Data>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogoutResponse {
    pub acknowledgement: Option<String>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Status {
    Success,
    Failure,
    Error,
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
impl ResponseEnumTrait for GetResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return self.data.clone().map(|v| serde_json::to_value(v).unwrap());
    }
}
impl ResponseEnumTrait for LoginResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return serde_json::to_value(self.status.clone()).ok();
    }
}
impl ResponseEnumTrait for SearchResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return self.data.clone().map(|v| serde_json::to_value(v).unwrap());
    }
}
impl ResponseEnumTrait for CreateResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return serde_json::to_value(self.status.clone()).ok();
    }
}
impl ResponseEnumTrait for LogoutResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return serde_json::to_value(self.status.clone()).ok();
    }
}
impl ResponseEnumTrait for ErrorResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        let mut data = Map::new();
        if let Some(message) = &self.message {
            data.insert("message".to_owned(), serde_json::to_value(message).unwrap());
        }
        if let Some(code) = &self.code {
            data.insert("code".to_owned(), serde_json::to_value(code).unwrap());
        }
        if data.is_empty() {
            return None;
        }
        Some(data.into())
    }
}
impl ResponseEnumTrait for InitResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return self.data.clone().map(|v| serde_json::to_value(v).unwrap());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[enum_dispatch(ResponseEnumTrait,Into<JsValue>)]
pub enum ResponseEnum {
    #[serde(rename = "get_response")]
    GetResponse(GetResponse),
    #[serde(rename = "search_response")]
    SearchResponse(SearchResponse),
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageEnum {
    Request(RequestEnum),
    Response(ResponseEnum),
}

#[enum_dispatch]
pub trait ResponseEnumTrait: Debug {
    fn get_acknowledgement(&self) -> Option<String>;
    fn get_data(&self) -> Option<serde_json::Value>;
    // fn get_response_type(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Data {
    String(String),
    JSON(Map<String, serde_json::Value>),
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
