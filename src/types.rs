use std::collections::HashMap;

use serde;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Account {
    pub username: String,
    pub id: String,
    pub domain: Option<String>,
    pub password: Option<String>,
    pub path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub custom_fields: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "username")]
    Username,
    #[serde(rename = "account")]
    Account,
    #[serde(rename = "auth")]
    Auth,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum StorageStatus {
    #[default]
    #[serde(rename = "Uninitialized")]
    Uninitialized,
    #[serde(rename = "loaded")]
    Loaded,
    #[serde(rename = "loading")]
    Loading(Option<String>),
    #[serde(rename = "error")]
    Error,
}
