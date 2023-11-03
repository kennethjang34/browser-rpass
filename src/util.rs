#![allow(non_upper_case_globals)]
use gloo::storage::errors::StorageError;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Promise;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;
use url::Url;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;
pub fn create_request_acknowledgement() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub async fn sleep(duration: Duration) {
    JsFuture::from(Promise::new(&mut |yes, _| {
        window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                &yes,
                duration.as_millis() as i32,
            )
            .unwrap();
    }))
    .await
    .unwrap();
}

pub async fn clipboard_copy(text: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().expect("Missing Window");
    let navigator = window.navigator();
    let clipboard = navigator.clipboard().expect("Missing Clipboard");
    let result = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(text)).await?;
    Ok(result)
}
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
    #[wasm_bindgen(js_name = "chrome.runtime.connectNative")]
    pub fn connect_native(s: &str) -> Port;
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
pub fn get_domain_name(addr: &String) -> String {
    let url = Url::parse(&addr).unwrap();
    let domain = url.domain().unwrap();
    let domain_name = domain.to_string();
    domain_name
}
unsafe impl Send for Port {}
unsafe impl Sync for Port {}
impl StorageArea {
    pub async fn get_all(&self) -> Result<JsValue, JsValue> {
        let key: JsValue = JsValue::NULL;
        Ok(chrome.storage().session().get(key.clone()).await)
    }
    pub async fn get_value(&self, key: &str) -> Result<JsValue, StorageError> {
        let js_key: JsValue = key.into();
        let entry = chrome.storage().session().get(js_key.clone()).await;
        js_sys::Reflect::get(&entry, &js_key).map_err(|_| StorageError::KeyNotFound(key.to_owned()))
    }
    pub async fn get_string_value(&self, key: &str) -> Result<Option<String>, StorageError> {
        let js_key: JsValue = key.into();
        let entry = chrome.storage().session().get(js_key.clone()).await;
        js_sys::Reflect::get(&entry, &js_key)
            .map(|v| {
                if let Some(v) = v.as_string() {
                    Some(v)
                } else {
                    None
                }
            })
            .map_err(|_| StorageError::KeyNotFound(key.to_owned()))
    }
    pub fn get_string_value_sync(&self, key: &str) -> Result<Option<String>, StorageError> {
        let js_key: JsValue = key.into();
        let entry = chrome.storage().session().get_sync(js_key.clone());
        js_sys::Reflect::get(&entry, &js_key)
            .map(|v| {
                if let Some(v) = v.as_string() {
                    Some(v)
                } else {
                    None
                }
            })
            .map_err(|_| StorageError::KeyNotFound(key.to_owned()))
    }
    pub fn set_string_item_sync(&self, key: String, value: String) {
        let mut entry = HashMap::new();
        entry.insert(key, value);
        let js_val = <JsValue as JsValueSerdeExt>::from_serde(&entry).unwrap();
        chrome.storage().session().set_sync(js_val);
    }
    pub async fn set_string_item(&self, key: String, value: String) {
        let mut entry = HashMap::new();
        entry.insert(key, value);
        let js_val = <JsValue as JsValueSerdeExt>::from_serde(&entry).unwrap();
        chrome.storage().session().set(js_val).await;
    }
    pub async fn set_item(&self, key: String, value: JsValue) {
        let entry = js_sys::Map::new();
        entry.set(&JsValue::from_str(&key), &value);
        chrome.storage().session().set(entry.into()).await;
    }
}
