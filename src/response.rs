use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetResponse {
    pub acknowledgement: Option<String>,
    pub data: Option<Data>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResponse {
    pub acknowledgement: Option<String>,
    pub data: Option<Vec<Data>>,
}
impl ResponseEnumTrait for GetResponse {
    fn get_acknowledgement(&self) -> Option<String> {
        return self.acknowledgement.clone();
    }
    fn get_data(&self) -> Option<serde_json::Value> {
        return self.data.clone().map(|v| serde_json::to_value(v).unwrap());
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorStruct {
    pub acknowledgement: Option<String>,
    pub description: String,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[enum_dispatch(ResponseEnumTrait)]
pub enum ResponseEnum {
    #[serde(rename = "get_response")]
    GetResponse(GetResponse),
    #[serde(rename = "search_response")]
    SearchResponse(SearchResponse),
}
// impl ResponseEnum{
//
// }

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
