use gloo_utils::format::JsValueSerdeExt;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

use crate::util;

use browser_rpass::request::*;

use util::*;

#[cfg(test)]
#[wasm_bindgen(start)]
pub async fn main() {
    let native_port = chrome.runtime().connect_native("com.rpass");
    let on_native_message_cb = Closure::<dyn Fn(String)>::new(move |msg: String| {
        log!("msg from native app : {:?}", msg);
    });
    native_port
        .on_message()
        .add_listener(on_native_message_cb.as_ref().clone());
    on_native_message_cb.forget();

    let mut init_request = HashMap::new();
    init_request.insert("type".to_owned(), "init".to_owned());
    init_request.insert("home_dir".to_owned(), "/Users/JANG".to_owned());
    let mut create_request = HashMap::new();
    create_request.insert("type".to_owned(), "create".to_owned());
    create_request.insert("username".to_owned(), "test_user".to_owned());
    create_request.insert("value".to_owned(), "create_test_password".to_owned());
    create_request.insert("path".to_owned(), "some.website.com".to_owned());
    let mut get_request_create_test = HashMap::new();
    get_request_create_test.insert("type".to_owned(), "get".to_owned());
    get_request_create_test.insert("username".to_owned(), "test_user".to_owned());
    get_request_create_test.insert("passphrase".to_owned(), "abcd".to_owned());
    get_request_create_test.insert("path".to_owned(), "some.website.com".to_owned());
    let mut edit_request = HashMap::new();
    edit_request.insert("type".to_owned(), "edit".to_owned());
    edit_request.insert("username".to_owned(), "test_user".to_owned());
    edit_request.insert("path".to_owned(), "some.website.com".to_owned());
    edit_request.insert("value".to_owned(), "........".to_owned());
    edit_request.insert("passphrase".to_owned(), "abcd".to_owned());
    let mut edit_request2 = HashMap::new();
    edit_request2.insert("type".to_owned(), "edit_username".to_owned());
    edit_request2.insert("username".to_owned(), "test_user".to_owned());
    edit_request2.insert("path".to_owned(), "some.website.com".to_owned());
    edit_request2.insert("value".to_owned(), "renamed_file2".to_owned());
    let mut create_request2 = HashMap::new();
    create_request2.insert("type".to_owned(), "create".to_owned());
    create_request2.insert("username".to_owned(), "test_user2".to_owned());
    create_request2.insert("value".to_owned(), "create_test_password2".to_owned());
    create_request2.insert("path".to_owned(), "some.website.com".to_owned());
    let mut delete_request = HashMap::new();
    delete_request.insert("type".to_owned(), "delete".to_owned());
    delete_request.insert("username".to_owned(), "test user".to_owned());
    delete_request.insert("path".to_owned(), "some.website.com".to_owned());
    native_port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&init_request).unwrap());

    let on_connect_cb = Closure::<dyn Fn(Port)>::new(move |port: Port| {
        let native_port = native_port.clone();
        let get_request_create_test = get_request_create_test.clone();
        let create_request = create_request.clone();
        let edit_request = edit_request.clone();
        let edit_request2 = edit_request2.clone();
        let create_request2 = create_request2.clone();
        let delete_request = delete_request.clone();
        let cb = Closure::<dyn Fn(JsValue, Port)>::new({
            move |msg: JsValue, port: Port| {
                log!("message received at service worker: {:?}", msg);
                log!("send another request to native host");
                let request: RequestEnum = <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                log!("request: {:?}", request);
                // log!("request_type: {:?}", request.request_type);
                // log!("payload: {:?}", request.payload);

                // let request: Request = <JsValue as JsValueSerdeExt>::into_serde(&msg).unwrap();
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&create_request.clone()).unwrap(),
                // );
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&get_request_create_test.clone())
                //         .unwrap(),
                // );
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&edit_request.clone()).unwrap(),
                // );
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&create_request2.clone()).unwrap(),
                // );
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&delete_request.clone()).unwrap(),
                // );
                // native_port.post_message(
                //     <JsValue as JsValueSerdeExt>::from_serde(&edit_request2.clone()).unwrap(),
                // );
            }
        });
        port.on_message().add_listener(cb.into_js_value());
        port.post_message("hello from service worker".into());
    });

    chrome
        .runtime()
        .on_connect()
        .add_listener(on_connect_cb.into_js_value());
}
