use enum_dispatch::enum_dispatch;
use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use serde::{Deserialize, Serialize};

use crate::util::create_request_acknowledgement;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename = "get")]
pub struct GetRequest {
    pub id: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename = "search")]
pub struct SearchRequest {
    pub query: String,
    pub resource: Resource,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename = "init")]
pub struct InitRequest {
    #[serde(flatten)]
    pub config: HashMap<String, String>,
    pub acknowledgement: Option<String>,
    #[serde(flatten)]
    pub header: Option<HashMap<String, String>>,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[enum_dispatch(RequestEnumTrait)]
pub enum RequestEnum {
    #[serde(rename = "get")]
    Get(GetRequest),
    #[serde(rename = "search")]
    Search(SearchRequest),
    #[serde(rename = "init")]
    Init(InitRequest),
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
