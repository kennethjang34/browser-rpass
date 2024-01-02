#![allow(non_upper_case_globals)]
pub use crate::js_binding;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use serde_json::Value;
use url::Url;

pub fn create_request_acknowledgement() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

pub fn get_domain_name(addr: &String) -> String {
    let url = Url::parse(&addr).unwrap();
    let domain = url.domain().unwrap();
    let domain_name = domain.to_string();
    domain_name
}

pub fn json_string_to_map(
    json_string: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, serde_json::Error> {
    serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(json_string)
}

pub trait JsonValueExt {
    fn get_string(&self, key: &str) -> Result<Option<&str>, String>;
    fn get_object(
        &self,
        key: &str,
    ) -> Result<Option<&serde_json::Map<String, serde_json::Value>>, String>;
    fn get_object_mut(
        &mut self,
        key: &str,
    ) -> Result<Option<&mut serde_json::Map<String, serde_json::Value>>, String>;
}

impl JsonValueExt for serde_json::Value {
    fn get_string(&self, key: &str) -> Result<Option<&str>, String> {
        let json_map = self
            .as_object()
            .ok_or("Passed serde_json::Value is not of Object")?;
        if let Some(value) = json_map.get(key) {
            if let Some(value_str) = value.as_str() {
                return Ok(Some(value_str));
            }
        }
        Ok(None)
    }
    fn get_object(
        &self,
        key: &str,
    ) -> Result<Option<&serde_json::Map<String, serde_json::Value>>, String> {
        let json_map = &self
            .as_object()
            .ok_or("Passed serde_json::Value is not of Object".to_string())?;
        if let Some(value) = json_map.get(key) {
            if let Some(value) = value.as_object() {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
    fn get_object_mut(
        &mut self,
        key: &str,
    ) -> Result<Option<&mut serde_json::Map<String, serde_json::Value>>, String> {
        let json_map = self
            .as_object_mut()
            .ok_or("Passed serde_json::Value is not of Object")?;
        if let Some(value) = json_map.get_mut(key) {
            if let Some(value) = value.as_object_mut() {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}
impl JsonValueExt for serde_json::Map<String, Value> {
    fn get_string(&self, key: &str) -> Result<Option<&str>, String> {
        if let Some(value) = self.get(key) {
            if let Some(value_str) = value.as_str() {
                return Ok(Some(value_str));
            }
        }
        Ok(None)
    }
    fn get_object(
        &self,
        key: &str,
    ) -> Result<Option<&serde_json::Map<String, serde_json::Value>>, String> {
        if let Some(value) = self.get(key) {
            if let Some(value) = value.as_object() {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
    fn get_object_mut(
        &mut self,
        key: &str,
    ) -> Result<Option<&mut serde_json::Map<String, serde_json::Value>>, String> {
        if let Some(value) = self.get_mut(key) {
            if let Some(value) = value.as_object_mut() {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
}

pub fn string_filter<'a, 'b>(options: Vec<&'a str>, query: &'b str) -> Vec<&'a str> {
    options
        .into_iter()
        .filter(|option| option.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}
