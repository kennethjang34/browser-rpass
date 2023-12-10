#![allow(non_upper_case_globals)]
pub use crate::js_binding;
use js_sys::Promise;
use std::time::Duration;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

pub mod extension_api;

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
