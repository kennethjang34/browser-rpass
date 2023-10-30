use browser_rpass::{create_request_acknowledgement, response::RequestEnum, types::Resource};
use gloo_utils::{document, format::JsValueSerdeExt};
use js_sys::Promise;
use log::{debug, info};
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, HtmlDataListElement, HtmlInputElement, HtmlOptionElement};
use yewdux::prelude::Dispatch;

use crate::store::{ContentScriptStore, DataAction, EXTENSION_PORT};
const USERNAME_INPUT_ELEMENT_ID_LIST: &[&str] = &[
    "username",
    "email",
    "user",
    "login",
    "account",
    "customer_email",
    "user_id",
    "id",
    "user_name",
    "user_email",
    "user",
    "identifierId",
];
const PASSWORD_INPUT_ELEMENT_ID_LIST: &[&str] =
    &["password", "pass", "pwd", "secret", "token", "auth"];

#[allow(dead_code)]
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

#[allow(unused_macros)]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn find_input_element_with_candidate_ids(id_list: &[&str]) -> Option<HtmlInputElement> {
    id_list
        .into_iter()
        .find_map(|candidate| document().get_element_by_id(candidate))
        .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
}
pub fn find_email_input_element() -> Option<HtmlInputElement> {
    let input_elements = document().get_elements_by_tag_name("input");
    let length = input_elements.length();
    for i in 0..length {
        if let Some(input_element) = input_elements.get_with_index(i) {
            let res = input_element
                .dyn_into::<HtmlInputElement>()
                .ok()
                .and_then(|input_element| {
                    info!("input_element.type_(): {}", input_element.type_());
                    if input_element.type_() == "email" {
                        debug!("find_email_input_element: email input element found");
                        Some(input_element)
                    } else {
                        None
                    }
                });
            if res.is_some() {
                return res;
            }
        }
    }
    debug!("no email input element found");
    return None;
}
pub fn find_password_input_element() -> Option<HtmlInputElement> {
    let input_elements = document().get_elements_by_tag_name("input");
    let length = input_elements.length();
    for i in 0..length {
        if let Some(input_element) = input_elements.get_with_index(i) {
            let res = input_element
                .dyn_into::<HtmlInputElement>()
                .ok()
                .and_then(|input_element| {
                    if input_element.type_() == "password" {
                        Some(input_element)
                    } else {
                        None
                    }
                });
            if res.is_some() {
                return res;
            }
        }
    }
    find_input_element_with_candidate_ids(PASSWORD_INPUT_ELEMENT_ID_LIST)
}
pub fn find_username_input_element() -> Option<HtmlInputElement> {
    find_input_element_with_candidate_ids(USERNAME_INPUT_ELEMENT_ID_LIST)
        .or(find_email_input_element())
}
pub fn _create_autocomplete_suggestion_element(
    id: &str,
    target_element: Option<&HtmlInputElement>,
) -> HtmlDataListElement {
    let suggestion_element = document()
        .create_element("datalist")
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    suggestion_element.set_id(id);
    if let Some(target_element) = target_element {
        target_element.set_autocomplete_suggestion_list(&id);
    }
    document()
        .body()
        .unwrap()
        .append_child(&suggestion_element)
        .unwrap();
    suggestion_element.unchecked_into()
}

pub trait HtmlInputElementExt {
    fn set_autocomplete_suggestion_list(&self, list: &str);
    fn set_autocomplete_suggestion_list_element(&self, list: &HtmlDataListElement);
}
impl HtmlInputElementExt for HtmlInputElement {
    fn set_autocomplete_suggestion_list(&self, list: &str) {
        self.set_attribute("list", list).unwrap();
    }
    fn set_autocomplete_suggestion_list_element(&self, list: &HtmlDataListElement) {
        self.set_autocomplete_suggestion_list(&list.id());
    }
}

pub trait HtmlDataListElementExt {
    fn add_option(&self, child: &str) -> HtmlOptionElement;
}
impl HtmlDataListElementExt for HtmlDataListElement {
    fn add_option(&self, value: &str) -> HtmlOptionElement {
        let option = document()
            .create_element("option")
            .unwrap()
            .dyn_into::<web_sys::HtmlOptionElement>()
            .unwrap();
        option.set_attribute("value", value).unwrap();
        self.append_child(&option).unwrap();
        option
    }
}
pub fn fetch_accounts(path: Option<String>) -> String {
    let dispatch = Dispatch::<ContentScriptStore>::new();
    let acknowledgement = create_request_acknowledgement();
    let fetch_request = RequestEnum::create_fetch_request(
        path,
        Resource::Account,
        Some(acknowledgement.clone()),
        None,
    );
    dispatch.apply(DataAction::ResourceFetchStarted(Resource::Account));
    EXTENSION_PORT
        .lock()
        .borrow()
        .post_message(<JsValue as JsValueSerdeExt>::from_serde(&fetch_request).unwrap());
    return acknowledgement;
}
