use std::{collections::HashMap, ops::Deref};

use crate::{
    api::types::Account,
    components::{account_entry_list::AccountEntryList, *},
    event_handlers::request_handlers::{create_response_listener, create_response_process_cb},
    pages::home_page::HomePage,
    pages::login_page::LoginPage,
    store::{PopupStore, EXTENSION_PORT},
};
use account_entry::AccountEntry;
use browser_rpass::{
    log,
    request::{RequestEnum, RequestEnumTrait, Resource},
    store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP,
    util::{chrome, create_request_acknowledgement},
};
use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::JsValue;
use yew::prelude::*;
use yewdux::prelude::{use_store, Dispatch};

#[function_component]
pub fn App() -> Html {
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
    log!("App");
    let (popup_store, popup_store_dispatch) = use_store::<PopupStore>();
    use_effect_with_deps(
        move |_| {
            let on_message_cb = create_response_listener(EXTENSION_PORT.clone());
            let port = EXTENSION_PORT.clone();
            port.on_message()
                .add_listener(on_message_cb.into_js_value());
            wasm_bindgen_futures::spawn_local(async move {
                if popup_store.verified {
                    let get_password_request = RequestEnum::create_get_request(
                        "some.website.com".to_owned(),
                        Resource::Password,
                        Some(create_request_acknowledgement()),
                        None,
                    );
                    let get_username_request = RequestEnum::create_search_request(
                        "some.website.com".to_owned(),
                        Resource::Username,
                        Some(create_request_acknowledgement()),
                        None,
                    );
                    let ctx = HashMap::new();
                    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
                        get_password_request.get_acknowledgement().clone().unwrap(),
                        create_response_process_cb(get_password_request.clone(), ctx),
                    );
                    let ctx = HashMap::new();
                    MESSAGE_ACKNOWLEDGEMENTS_POP_UP.lock().unwrap().insert(
                        get_username_request.get_acknowledgement().clone().unwrap(),
                        create_response_process_cb(get_username_request.clone(), ctx),
                    );
                    port.post_message(
                        <JsValue as JsValueSerdeExt>::from_serde(&get_password_request).unwrap(),
                    );
                    port.post_message(
                        <JsValue as JsValueSerdeExt>::from_serde(&get_username_request).unwrap(),
                    );
                } else {
                    log!("not verified");
                    // }
                }
            });
        },
        (),
    );

    html! {
        <HomePage/>
    }
}
