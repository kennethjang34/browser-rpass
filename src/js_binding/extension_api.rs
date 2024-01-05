use gloo::storage::errors::StorageError;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Promise;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::store;
#[wasm_bindgen]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type Port;
    #[derive(Debug)]
    pub type Runtime;
    #[derive(Debug)]
    pub type Chrome;
    #[derive(Debug)]
    pub type EventTarget;
    #[derive(Debug, Clone, PartialEq)]
    pub type Tabs;
    #[derive(Debug, Clone, PartialEq)]
    pub type Tab;
    #[derive(Debug)]
    pub type MessageSender;
    #[derive(Debug)]
    pub type Event;
    #[derive(Debug)]
    pub type Storage;
    #[derive(Debug)]
    pub type StorageArea;

    #[wasm_bindgen(js_name = "chrome")]
    pub static chrome: Chrome;

    #[wasm_bindgen(method, getter=runtime,structural,js_name=runtime)]
    pub fn runtime(this: &Chrome) -> Runtime;
    #[wasm_bindgen(method, getter=tabs,structural,js_name=tabs)]
    pub fn tabs(this: &Chrome) -> Tabs;
    #[wasm_bindgen(method, getter=storage,structural,js_name=storage)]
    pub fn storage(this: &Chrome) -> Storage;
    #[wasm_bindgen(method,js_name=query)]
    pub fn query(this: &Tabs, query_info: JsValue) -> Promise;
    #[wasm_bindgen(method,getter=active)]
    pub fn active(this: &Tab) -> bool;
    #[wasm_bindgen(method,getter=discarded)]
    pub fn discarded(this: &Tab) -> bool;
    #[wasm_bindgen(method,getter=groupId)]
    pub fn group_id(this: &Tab) -> i32;
    #[wasm_bindgen(method,getter=id)]
    pub fn id(this: &Tab) -> i32;
    #[wasm_bindgen(method,getter=index)]
    pub fn index(this: &Tab) -> i32;
    #[wasm_bindgen(method,getter=url)]
    pub fn url(this: &Tab) -> Option<String>;
    #[wasm_bindgen(method,getter=windowId)]
    pub fn window_id(this: &Tab) -> i32;
    #[wasm_bindgen(method,getter=onConnect)]
    pub fn on_connect(this: &Runtime) -> EventTarget;
    #[wasm_bindgen(method,getter=session)]
    pub fn session(this: &Storage) -> StorageArea;
    #[wasm_bindgen(method,getter=local)]
    pub fn local(this: &Storage) -> StorageArea;
    #[wasm_bindgen(method,getter=sync)]
    pub fn sync(this: &Storage) -> StorageArea;
    #[wasm_bindgen(method,getter=onDisconnect)]
    pub fn on_disconnect(this: &Runtime) -> EventTarget;
    #[wasm_bindgen(method,getter=onDisconnect,structural)]
    pub fn on_disconnect(this: &Port) -> EventTarget;
    #[wasm_bindgen(method,getter=name)]
    pub fn name(this: &Port) -> String;
    #[wasm_bindgen(method,setter=name)]
    pub fn set_name(this: &Port, val: String) -> String;
    #[wasm_bindgen(method, getter=onMessage)]
    pub fn on_message(this: &Port) -> EventTarget;
    #[wasm_bindgen(method,js_name=disconnect)]
    pub fn disconnect(this: &Port);
    #[wasm_bindgen(method, getter=sender,structural)]
    pub fn sender(this: &Port) -> Option<MessageSender>;

    #[wasm_bindgen(method, getter=onMessage,structural)]
    pub fn on_message(this: &Runtime) -> EventTarget;
    #[wasm_bindgen(structural, method, js_name = "addListener")]
    pub fn add_listener(this: &EventTarget, callback: JsValue);
    #[wasm_bindgen(catch, js_name = "chrome.runtime.connectNative")]
    pub fn connect_native(s: &str) -> Result<Port, JsValue>;
    #[wasm_bindgen(method,structural,js_name=connectNative)]
    pub fn connect_native(this: &Runtime, s: &str) -> Port;
    #[wasm_bindgen(method,structural,js_name=connect)]
    pub fn connect_exteranl(this: &Runtime, s: &str) -> Port;
    #[wasm_bindgen(method,structural,js_name=connect)]
    pub fn connect(this: &Runtime) -> Port;
    #[wasm_bindgen(method, js_class = "Port", js_name = "postMessage")]
    pub fn post_message(this: &Port, message: JsValue);
    #[wasm_bindgen(method,structural,js_name=sendNativeMessage)]
    pub fn send_native_message(
        this: &Runtime,
        target: &str,
        message: JsValue,
        callback: Option<&Closure<dyn Fn(String)>>,
    ) -> Port;
    #[wasm_bindgen(js_namespace = chrome, js_name = "runtime.sendNativeMessage")]
    pub async fn send_native_message(
        target: &str,
        message: JsValue,
        callback: Option<&Closure<dyn Fn(String)>>,
    );
    #[wasm_bindgen(method,structural,js_name=get)]
    pub async fn get(this: &StorageArea, key: JsValue) -> JsValue;
    #[wasm_bindgen(method, structural, js_name = "set")]
    pub async fn set(this: &StorageArea, items: JsValue);
    #[wasm_bindgen(method,structural,js_name=get)]
    pub fn get_sync(this: &StorageArea, key: JsValue) -> JsValue;
    #[wasm_bindgen(method, structural, js_name = "set")]
    pub fn set_sync(this: &StorageArea, items: JsValue);
    #[wasm_bindgen(method, structural, js_name = "remove")]
    pub async fn remove(this: &StorageArea, key: &str) -> JsValue;
    #[wasm_bindgen(method, structural, js_name = "remove")]
    pub async fn remove_bulk(this: &StorageArea, key: JsValue) -> JsValue;
}
unsafe impl Send for Port {}
unsafe impl Sync for Port {}
impl StorageArea {
    pub async fn get_all(&self, area: store::StorageArea) -> Result<JsValue, JsValue> {
        let key: JsValue = JsValue::NULL;
        let fetched = {
            match area {
                store::StorageArea::Local => chrome.storage().local(),
                store::StorageArea::Sync => chrome.storage().sync(),
                store::StorageArea::Session => chrome.storage().session(),
            }
        }
        .get(key.clone())
        .await;
        Ok(fetched)
    }
    pub async fn get_value(
        &self,
        key: &str,
        storage: store::StorageArea,
    ) -> Result<JsValue, StorageError> {
        let js_key: JsValue = key.into();
        let entry = {
            let storage = match storage {
                store::StorageArea::Local => chrome.storage().local(),
                store::StorageArea::Sync => chrome.storage().sync(),
                store::StorageArea::Session => chrome.storage().session(),
            };
            storage.get(js_key.clone()).await
        };
        if entry.is_undefined() {
            return Err(StorageError::KeyNotFound(key.to_owned()));
        } else {
            return Ok(entry);
        }
        // entry.map_err(|_| StorageError::KeyNotFound(key.to_owned()))
    }
    pub fn get_all_sync(&self, storage: store::StorageArea) -> JsValue {
        let key: JsValue = JsValue::NULL;
        let fetched = match storage {
            store::StorageArea::Local => chrome.storage().local().get_sync(key.clone()),
            store::StorageArea::Sync => chrome.storage().sync().get_sync(key.clone()),
            store::StorageArea::Session => chrome.storage().session().get_sync(key.clone()),
        };
        fetched
    }
    pub async fn get_item(
        &self,
        key: &str,
        storage: store::StorageArea,
    ) -> Result<JsValue, StorageError> {
        let entry = {
            let storage = {
                match storage {
                    store::StorageArea::Local => chrome.storage().local(),
                    store::StorageArea::Sync => chrome.storage().sync(),
                    store::StorageArea::Session => chrome.storage().session(),
                }
            };
            storage.get(key.into()).await
        };
        let value = js_sys::Reflect::get(&entry, &JsValue::from_str(key));
        if let Ok(value) = value {
            return Ok(value);
        } else {
            return Err(StorageError::KeyNotFound(key.to_owned()));
        }
    }
    pub fn get_string_value_sync(
        &self,
        key: &str,
        storage: store::StorageArea,
    ) -> Result<Option<String>, StorageError> {
        let js_key: JsValue = key.into();
        let entry = {
            match storage {
                store::StorageArea::Local => chrome.storage().local().get_sync(js_key.clone()),
                store::StorageArea::Sync => chrome.storage().sync().get_sync(js_key.clone()),
                store::StorageArea::Session => chrome.storage().session().get_sync(js_key.clone()),
            }
        };
        let _fut = js_sys::Reflect::get(&entry, &JsValue::from_str(&"PromiseResult")).unwrap();
        Ok(None)
    }
    pub fn set_string_item_sync(&self, key: String, value: String, stroage: store::StorageArea) {
        let mut entry = HashMap::new();
        entry.insert(key, value);
        let js_val = <JsValue as JsValueSerdeExt>::from_serde(&entry).unwrap();
        match stroage {
            store::StorageArea::Local => chrome.storage().local().set_sync(js_val),
            store::StorageArea::Sync => chrome.storage().sync().set_sync(js_val),
            store::StorageArea::Session => chrome.storage().session().set_sync(js_val),
        }
    }
    pub async fn set_string_item(&self, key: String, value: String, storage: store::StorageArea) {
        let mut entry = HashMap::new();
        entry.insert(key, value);
        let js_val = <JsValue as JsValueSerdeExt>::from_serde(&entry).unwrap();
        match storage {
            store::StorageArea::Local => chrome.storage().local().set(js_val).await,
            store::StorageArea::Sync => chrome.storage().sync().set(js_val).await,
            store::StorageArea::Session => chrome.storage().session().set(js_val).await,
        }
    }
}
