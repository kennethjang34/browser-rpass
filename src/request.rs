use core::fmt;
use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
use serde_json::Value;
use serde_variant::to_variant_name;
use std::collections::HashMap;
use std::string::ToString;
use strum_macros::Display;
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionEventType {
    Create,
    Update,
    Login,
    Logout,
    LoginError,
    Delete,
    Search,
    Init,
    Error,
    Refreshed,
    CreationFailed,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SessionEvent {
    pub data: Option<Value>,
    pub event_type: SessionEventType,
    pub meta: Option<Value>,
    pub resource: Option<Vec<Resource>>,
    pub is_global: bool,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SessionEventWrapper {
    pub acknowledgement: Option<String>,
    pub session_event: SessionEvent,
    pub header: Option<HashMap<String, String>>,
}

use crate::{types::Resource, util::create_request_acknowledgement};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "get")]
pub struct GetRequest {
    pub id: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "edit")]
pub struct EditRequest {
    pub id: String,
    pub resource: Resource,
    pub domain: Option<String>,
    pub value: Value,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "delete")]
pub struct DeleteRequest {
    pub id: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "search")]
pub struct SearchRequest {
    pub query: Option<String>,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "fetch")]
pub struct FetchRequest {
    pub path: Option<String>,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "login")]
pub struct LoginRequest {
    pub username: String,
    pub passphrase: String,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "logout")]
pub struct LogoutRequest {
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "init")]
pub struct InitRequest {
    #[serde(flatten)]
    pub config: HashMap<String, String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "create")]
pub struct CreateRequest {
    pub username: String,
    pub resource: Resource,
    pub domain: String,
    pub value: Value,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
macro_rules! request_enum_trait_impl {
    ($($t:ty)*) => ($(
        impl RequestEnumTrait for $t {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn set_acknowledgement(&mut self, acknowledgement:String) {
        self.acknowledgement=Some(acknowledgement);
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
        }

    )*)
}
request_enum_trait_impl!(GetRequest);
request_enum_trait_impl!(SearchRequest);
request_enum_trait_impl!(FetchRequest);
request_enum_trait_impl!(LoginRequest);
request_enum_trait_impl!(LogoutRequest);
request_enum_trait_impl!(InitRequest);
request_enum_trait_impl!(CreateRequest);
request_enum_trait_impl!(DeleteRequest);
request_enum_trait_impl!(EditRequest);
request_enum_trait_impl!(SessionEventWrapper);
macro_rules! into_js_value_impl {
    ($($t:ty)*) => ($(
        impl Into<JsValue> for $t {
            #[inline]
            fn into(self)->JsValue {
                <JsValue as JsValueSerdeExt>::from_serde(&self).unwrap()
            }
        }

    )*)
}
into_js_value_impl!(GetRequest);
into_js_value_impl!(SearchRequest);
into_js_value_impl!(FetchRequest);
into_js_value_impl!(LoginRequest);
into_js_value_impl!(LogoutRequest);
into_js_value_impl!(InitRequest);
into_js_value_impl!(CreateRequest);
into_js_value_impl!(DeleteRequest);
into_js_value_impl!(EditRequest);
into_js_value_impl!(SessionEvent);
into_js_value_impl!(SessionEventWrapper);

#[derive(Serialize, Deserialize, Debug, Clone, Display, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[enum_dispatch(RequestEnumTrait)]
pub enum RequestEnum {
    #[serde(rename = "get")]
    Get(GetRequest),
    #[serde(rename = "delete")]
    Delete(DeleteRequest),
    #[serde(rename = "edit")]
    Edit(EditRequest),
    #[serde(rename = "search")]
    Search(SearchRequest),
    #[serde(rename = "fetch")]
    Fetch(FetchRequest),
    #[serde(rename = "login")]
    Login(LoginRequest),
    #[serde(rename = "logout")]
    Logout(LogoutRequest),
    #[serde(rename = "init")]
    Init(InitRequest),
    #[serde(rename = "create")]
    Create(CreateRequest),
    #[serde(rename = "session_event")]
    SessionEventRequest(SessionEventWrapper),
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
    pub fn create_delete_request(
        id: String,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Delete(DeleteRequest {
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
        resource: Resource,
        value: Value,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Create(CreateRequest {
            username,
            domain: path,
            resource,
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
        query: Option<String>,
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
    pub fn create_fetch_request(
        path: Option<String>,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Fetch(FetchRequest {
            path,
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
        user_id: String,
        passphrase: String,
    ) -> RequestEnum {
        RequestEnum::Login(LoginRequest {
            username: user_id,
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
    pub fn create_edit_request(
        id: String,
        resource: Resource,
        domain: Option<String>,
        value: Value,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Edit(EditRequest {
            id,
            resource,
            domain,
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
    pub fn create_session_event_request(
        acknowledgement: Option<String>,
        session_event: SessionEvent,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::SessionEventRequest(SessionEventWrapper {
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            session_event,
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
    pub fn get_type(&self) -> String {
        self.to_string()
    }
}

#[enum_dispatch]
pub trait RequestEnumTrait {
    fn get_acknowledgement(&self) -> Option<String>;
    fn get_header(&self) -> Option<HashMap<String, String>>;
    fn set_header(&mut self, header: HashMap<String, String>);
    fn set_acknowledgement(&mut self, acknowledgement: String);
}
impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_variant_name(&self).unwrap())
    }
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
