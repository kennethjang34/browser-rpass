use core::fmt;
use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
#[allow(unused_imports)]
use log::debug;
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
    NativeAppConnectionError,
    Update,
    Refreshed,
    Delete,
    CreationFailed,
    Login,
    Logout,
    LogoutError,
    LoginError,
    Search,
    Init(HashMap<DataFieldType, Value>),
    Error,
    CreateStore,
    StoreCreated(HashMap<DataFieldType, Value>, String),
    StoreCreationFailed(HashMap<DataFieldType, Value>, String),
    StoreDeleted(HashMap<DataFieldType, Value>, String),
    StoreDeletionFailed(HashMap<DataFieldType, Value>, String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum DataFieldType {
    ErrorType,
    InitError,
    NativeAppConnectionError,
    CanSign,
    CanEncrypt,
    Keys,
    KeyHasSecret,
    KeyID,
    KeyFingerprint,
    KeyUserID,
    KeyUsable,
    KeyUsername,
    HomeDir,
    IsRepo,
    SubStore,
    IsDefault,
    DefaultStoreID,
    DefaultStoreAvailable,
    ContentScript,
    Request,
    SigningKey,
    StoreDir,
    StoreID,
    StorePath,
    ResourceID,
    UserID,
    Username,
    Passphrase,
    Password,
    Note,
    Recipient,
    ValidSignerList,
    RepoSigningKey,
    CustomField,
    Domain,
    Path,
    Resource,
    Query,
    Value,
    Verified,
    Error,
    ErrorMessage,
    ErrorCode,
    ErrorSource,
    Update,
    UpdatedFields,
    Delete,
    Create,
    Search,
    Fetch,
    Login,
    Logout,
    Status,
    Acknowledgement,
    Data,
    CreateStore,
    StoreIDList,
    ParentStoreId,
}
impl fmt::Display for DataFieldType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", to_variant_name(&self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SessionEvent {
    pub data: Option<HashMap<DataFieldType, Value>>,
    pub event_type: SessionEventType,
    pub meta: Option<Value>,
    pub resource: Option<Vec<Resource>>,
    pub is_global: bool,
    pub acknowledgement: Option<String>,
    pub store_id_index: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct SessionEventWrapper {
    pub session_event: SessionEvent,
}

use crate::{types::Resource, util::create_request_acknowledgement};
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "get")]
pub struct GetRequest {
    pub id: String,
    pub store_id: Option<String>,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "edit")]
pub struct EditRequest {
    pub id: String,
    pub store_id: Option<String>,
    pub resource: Resource,
    pub domain: Option<String>,
    pub value: HashMap<DataFieldType, Value>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "delete")]
pub struct DeleteRequest {
    pub id: String,
    pub store_id: Option<String>,
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
    pub store_id: Option<String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "fetch")]
pub struct FetchRequest {
    pub store_id: Option<String>,
    pub path: Option<String>,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "login")]
pub struct LoginRequest {
    pub store_id: Option<String>,
    pub prev_store_id: Option<String>,
    pub is_default: bool,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "logout")]
pub struct LogoutRequest {
    pub acknowledgement: Option<String>,
    pub store_id: Option<String>,
    pub user_id: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "init")]
pub struct InitRequest {
    #[serde(flatten)]
    pub config: HashMap<DataFieldType, String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
    store_id: Option<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "create")]
pub struct CreateRequest {
    pub username: Option<String>,
    pub resource: Resource,
    pub store_id: Option<String>,
    pub note: Option<String>,
    pub custom_fields: Option<HashMap<String, Value>>,
    pub domain: Option<String>,
    pub password: Option<String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "create_store")]
pub struct CreateStoreRequest {
    pub parent_store: Option<String>,
    pub store_name: String,
    pub encryption_keys: Vec<String>,
    pub valid_signing_keys: Option<Vec<String>>,
    pub repo_signing_key: Option<String>,
    pub is_repo: bool,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", rename = "delete_store")]
pub struct DeleteStoreRequset {
    pub store_id: String,
    pub acknowledgement: Option<String>,
    pub force: bool,
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
    fn get_store_id(&self) -> Option<String> {
        self.store_id.clone()
    }
        }

    )*)
}

impl CreateStoreRequest {
    pub fn get_store_path(&self) -> Option<String> {
        self.parent_store.clone()
    }
    pub fn get_store_name(&self) -> String {
        self.store_name.clone()
    }
    pub fn get_store_dir(&self) -> Option<String> {
        self.parent_store.clone()
    }
}

impl RequestEnumTrait for CreateStoreRequest {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn set_acknowledgement(&mut self, acknowledgement: String) {
        self.acknowledgement = Some(acknowledgement);
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_store_id(&self) -> Option<String> {
        Some(self.store_name.clone())
    }
}
impl RequestEnumTrait for DeleteStoreRequset {
    fn get_acknowledgement(&self) -> Option<String> {
        self.acknowledgement.clone()
    }
    fn set_acknowledgement(&mut self, acknowledgement: String) {
        self.acknowledgement = Some(acknowledgement);
    }
    fn get_header(&self) -> Option<HashMap<String, String>> {
        self.header.clone()
    }
    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = Some(header);
    }
    fn get_store_id(&self) -> Option<String> {
        Some(self.store_id.clone())
    }
}

// trait StoreID {
//     fn get_store_id(&self) -> String;
// }
// macro_rules! store_id_impl {
//     ($($t:ty)*) => ($(
//         impl StoreID for $t {
//             fn get_store_id(&self) -> String {
//                 self.store_id.clone()
//             }
//         }
//
//     )*)
// }

// store_id_impl!(FetchRequest);
// store_id_impl!(LoginRequest);
// store_id_impl!(LogoutRequest);
// store_id_impl!(CreateRequest);

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
into_js_value_impl!(DeleteStoreRequset);
into_js_value_impl!(InitRequest);
into_js_value_impl!(CreateRequest);
into_js_value_impl!(CreateStoreRequest);
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
    #[serde(rename = "create_store")]
    CreateStore(CreateStoreRequest),
    #[serde(rename = "delete_store")]
    DeleteStore(DeleteStoreRequset),
    #[serde(rename = "session_event")]
    SessionEventRequest(SessionEventWrapper),
}
impl RequestEnum {
    pub fn create_get_request(
        id: String,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
        store_id: Option<String>,
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
            store_id,
        })
    }
    pub fn create_delete_request(
        id: String,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
        store_id: Option<String>,
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
            store_id,
        })
    }
    pub fn create_create_request(
        store_id: Option<String>,
        username: Option<String>,
        domain: Option<String>,
        note: Option<String>,
        custom_fields: Option<HashMap<String, Value>>,
        resource: Resource,
        password: Option<String>,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Create(CreateRequest {
            store_id,
            username,
            domain,
            resource,
            note,
            custom_fields,
            password,
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
        store_id: Option<String>,
    ) -> RequestEnum {
        RequestEnum::Search(SearchRequest {
            store_id,
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
        store_id: Option<String>,
        path: Option<String>,
        resource: Resource,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::Fetch(FetchRequest {
            path,
            resource,
            store_id,
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
        store_id: Option<String>,
        prev_store_id: Option<String>,
        is_default: bool,
    ) -> RequestEnum {
        RequestEnum::Login(LoginRequest {
            is_default,
            store_id,
            prev_store_id,
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
        store_id: Option<String>,
        user_id: Option<String>,
    ) -> RequestEnum {
        RequestEnum::Logout(LogoutRequest {
            acknowledgement: {
                if acknowledgement.is_some() {
                    acknowledgement
                } else {
                    Some(create_request_acknowledgement())
                }
            },
            store_id,
            user_id,
            header,
        })
    }
    pub fn create_edit_request(
        id: String,
        resource: Resource,
        domain: Option<String>,
        value: HashMap<DataFieldType, Value>,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
        store_id: Option<String>,
    ) -> RequestEnum {
        RequestEnum::Edit(EditRequest {
            id,
            resource,
            domain,
            store_id,
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
        store_id: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::SessionEventRequest(SessionEventWrapper { session_event })
    }
    pub fn create_init_request(
        config: HashMap<DataFieldType, String>,
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
            store_id: None,
        })
    }
    pub fn create_create_store_request(
        parent_store: Option<String>,
        store_name: String,
        encryption_keys: Vec<String>,
        signing_keys: Option<Vec<String>>,
        is_repo: bool,
        repo_signing_key: Option<String>,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::CreateStore(CreateStoreRequest {
            parent_store,
            store_name,
            encryption_keys,
            valid_signing_keys: signing_keys,
            is_repo,
            repo_signing_key,
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
    pub fn create_delete_store_request(
        store_id: String,
        force: bool,
        acknowledgement: Option<String>,
        header: Option<HashMap<String, String>>,
    ) -> RequestEnum {
        RequestEnum::DeleteStore(DeleteStoreRequset {
            store_id,
            force,
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
    fn get_store_id(&self) -> Option<String>;
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
