use crate::{
    event_handlers::request_handlers::create_response_process_cb,
    pages::login_page::LoginPage,
    store::{LoginAction, PopupAction, PopupStore, EXTENSION_PORT},
};
use std::{collections::HashMap, ops::Deref};

use crate::{
    api::types::Account,
    components::{account_entry_list::AccountEntryList, header::Header},
};
use browser_rpass::{
    log, request::*, store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP, util::create_request_acknowledgement,
};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::{use_selector, use_store, Dispatch};

#[derive(Properties, PartialEq)]
pub struct Props {}
#[function_component(HomePage)]
pub fn home_page(props: &Props) -> Html {
    log!("home_page");
    let mock_accounts = vec![
        Account {
            id: 1,
            username: Some("mu1".to_owned()),
            email: "mu1@gmail.com".to_owned(),
            password: Some("abc".to_owned()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            organization: Some("apple company".to_owned()),
        },
        Account {
            id: 2,
            username: None,
            email: "mu2@gmail.com".to_owned(),
            password: Some("def".to_owned()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            organization: Some("banana company".to_owned()),
        },
    ];
    let verified = use_selector(|state: &PopupStore| state.verified);
    let (popup_store, popup_store_dispatch) = use_store::<PopupStore>();
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
    let password_input_ref2 = password_input_ref.clone();
    let username_input_ref2 = username_input_ref.clone();
    let on_create_submit = Callback::from(move |event: SubmitEvent| {
        event.prevent_default();
        let password_input = password_input_ref2.cast::<HtmlInputElement>().unwrap();
        let username_input = username_input_ref2.cast::<HtmlInputElement>().unwrap();
        let create_request = RequestEnum::create_create_request(
            username_input.value(),
            "some.website.com".to_owned(),
            Some(password_input.value()),
            Some(create_request_acknowledgement()),
            None,
        );
        let ctx = HashMap::new();
        MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
            create_request.get_acknowledgement().clone().unwrap(),
            create_response_process_cb(create_request.clone(), ctx),
        );
        EXTENSION_PORT
            .post_message(<JsValue as JsValueSerdeExt>::from_serde(&create_request).unwrap());
        username_input.set_value("");
        password_input.set_value("");
    });
    let on_logout_click = Callback::from(move |_| {
        let logout_request = RequestEnum::create_logout_request(None, None);
        let ctx = HashMap::new();
        MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
            logout_request.get_acknowledgement().clone().unwrap(),
            create_response_process_cb(logout_request.clone(), ctx),
        );
        EXTENSION_PORT
            .post_message(<JsValue as JsValueSerdeExt>::from_serde(&logout_request).unwrap());
        popup_store_dispatch.apply(LoginAction::LogoutStarted);
    });
    html! {
        <div>
            if popup_store.verified{
                <p class="mb-4">{format!("verified!!!!")}</p>
                <label for="account-search">{"Search for account:"}</label><br/>
                <input type="search" id="account-search" name="account-search"/>
            <button>{ "Search" }</button>
            <table class="table table-bordered">
                <thead>
                  <tr>
                    <th>{ "ID" }</th>
                    <th>{ "Password" }</th>
                  </tr>
                </thead>
                <tbody>
                <AccountEntryList accounts={mock_accounts.clone()}/>
                </tbody>
            </table>
            <div>
                <button onclick={on_logout_click}>{ "Logout" }</button>
                <form  onsubmit={on_create_submit}>
                <label for="new-username">{"new username:"}</label>
                <input type="text" id="new-username" name="create-username" ref={username_input_ref}/><br/>
                <label for="new-password">{"new password:"}</label>
                <input type="password" id="new-password" name="create-password" ref={password_input_ref}/><br/>
                <button>{ "Create" }</button>
                </form>
            </div>
            }else{
                <p>{"need to login!"}</p>
                    <LoginPage />
            }
        </div>
    }
}
