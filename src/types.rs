use std::collections::HashMap;

use secrecy::Secret;
use serde;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::request::DataFieldType;
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Account {
    pub username: String,
    pub id: String,
    pub domain: Option<String>,
    password: Option<String>,
    pub path: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
    #[serde(flatten)]
    pub custom_fields: Option<HashMap<String, Value>>,
}
impl Account {
    pub fn get_password(&self) -> Option<Secret<String>> {
        if let Some(password) = &self.password {
            Some(Secret::new(password.clone()))
        } else {
            None
        }
    }
    pub fn set_password(&mut self, password: Option<String>) {
        self.password = password;
    }
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
    #[serde(rename = "store")]
    Store,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum StateStoreStatus {
    #[default]
    #[serde(rename = "Uninitialized")]
    Uninitialized,
    #[serde(rename = "loaded")]
    Loaded,
    #[serde(rename = "loading")]
    Loading(Option<String>),
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "idle")]
    Idle,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct Key {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub has_secret: bool,
    pub is_usable: bool,
    pub can_sign: bool,
    pub can_encrypt: bool,
}
impl Key {
    pub fn new(id: String, name: Option<String>, email: Option<String>) -> Self {
        Self {
            id,
            name,
            email,
            has_secret: false,
            is_usable: false,
            can_sign: false,
            can_encrypt: false,
        }
    }
}
impl From<&Key> for String {
    fn from(key: &Key) -> Self {
        key.id.clone()
    }
}
impl From<Key> for Value {
    fn from(key: Key) -> Self {
        let mut map = HashMap::new();
        let fingerprint = key.id.clone();
        map.insert(
            DataFieldType::KeyFingerprint,
            serde_json::to_value(fingerprint.clone()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyID,
            serde_json::to_value(fingerprint).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsername,
            serde_json::to_value(key.name).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUserID,
            serde_json::to_value(key.email).unwrap(),
        );
        map.insert(
            DataFieldType::KeyHasSecret,
            serde_json::to_value(key.has_secret).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsable,
            serde_json::to_value(key.is_usable).unwrap(),
        );
        map.insert(
            DataFieldType::CanSign,
            serde_json::to_value(key.can_sign).unwrap(),
        );
        map.insert(
            DataFieldType::CanEncrypt,
            serde_json::to_value(key.can_encrypt).unwrap(),
        );
        serde_json::to_value(map).unwrap()
    }
}
impl From<&Key> for Value {
    fn from(key: &Key) -> Self {
        let mut map = HashMap::new();
        let fingerprint = key.id.clone();
        map.insert(
            DataFieldType::KeyFingerprint,
            serde_json::to_value(fingerprint.clone()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyID,
            serde_json::to_value(fingerprint).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsername,
            serde_json::to_value(key.name.clone()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUserID,
            serde_json::to_value(key.email.clone()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyHasSecret,
            serde_json::to_value(key.has_secret).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsable,
            serde_json::to_value(key.is_usable).unwrap(),
        );
        map.insert(
            DataFieldType::CanSign,
            serde_json::to_value(key.can_sign).unwrap(),
        );
        map.insert(
            DataFieldType::CanEncrypt,
            serde_json::to_value(key.can_encrypt).unwrap(),
        );
        serde_json::to_value(map).unwrap()
    }
}
impl From<Value> for Key {
    fn from(map: Value) -> Self {
        let map: HashMap<DataFieldType, Value> = serde_json::from_value(map).unwrap();
        Key {
            id: map
                .get(&DataFieldType::KeyID)
                .map(|v| v.as_str().unwrap().to_string())
                .unwrap(),
            name: map
                .get(&DataFieldType::KeyUsername)
                .map(|v| v.as_str().unwrap().to_string()),
            email: map
                .get(&DataFieldType::KeyUserID)
                .map(|v| v.as_str().unwrap().to_string()),
            has_secret: map
                .get(&DataFieldType::KeyHasSecret)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            is_usable: map
                .get(&DataFieldType::KeyUsable)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            can_sign: map
                .get(&DataFieldType::CanSign)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            can_encrypt: map
                .get(&DataFieldType::CanEncrypt)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
        }
    }
}
impl From<&HashMap<DataFieldType, Value>> for Key {
    fn from(map: &HashMap<DataFieldType, Value>) -> Self {
        Key {
            id: map
                .get(&DataFieldType::KeyID)
                .map(|v| v.as_str().unwrap().to_string())
                .unwrap(),
            name: map
                .get(&DataFieldType::KeyUsername)
                .map(|v| v.as_str().unwrap().to_string()),
            email: map
                .get(&DataFieldType::KeyUserID)
                .map(|v| v.as_str().unwrap().to_string()),
            has_secret: map
                .get(&DataFieldType::KeyHasSecret)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            is_usable: map
                .get(&DataFieldType::KeyUsable)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            can_sign: map
                .get(&DataFieldType::CanSign)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
            can_encrypt: map
                .get(&DataFieldType::CanEncrypt)
                .map(|v| v.as_bool().unwrap())
                .unwrap(),
        }
    }
}
