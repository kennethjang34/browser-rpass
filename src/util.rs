use gloo_utils::format::JsValueSerdeExt;
use js_sys::Promise;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;
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
    #[derive(Debug, Clone)]
    pub type Port;
    #[derive(Debug)]
    pub type Runtime;
    #[derive(Debug)]
    pub type Chrome;
    #[derive(Debug)]
    pub type EventTarget;
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
    #[wasm_bindgen(method, getter=storage,structural,js_name=storage)]
    pub fn storage(this: &Chrome) -> Storage;

    // #[wasm_bindgen(js_namespace=console,js_name=log)]
    // pub fn log(object: &JsValue);

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
    #[wasm_bindgen(method, structural, js_name = "remove")]
    pub async fn remove(this: &StorageArea, key: &str) -> JsValue;
    #[wasm_bindgen(method, structural, js_name = "remove")]
    pub async fn remove_bulk(this: &StorageArea, key: JsValue) -> JsValue;
}
unsafe impl Send for Port {}
unsafe impl Sync for Port {}
impl StorageArea {
    pub async fn get_value(&self, key: &str) -> Result<JsValue, JsValue> {
        let key: JsValue = key.into();
        let entry = chrome.storage().session().get(key.clone()).await;
        js_sys::Reflect::get(&entry, &key)
    }
    pub async fn set_value(&self, key: String, value: String) {
        let mut entry = HashMap::new();
        entry.insert(key, value);
        let js_val = <JsValue as JsValueSerdeExt>::from_serde(&entry).unwrap();
        chrome.storage().session().set(js_val).await;
    }
}
