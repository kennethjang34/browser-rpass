use std::{collections::HashMap, ops::Deref};

use crate::{
    api::types::Account,
    components::{account_entry_list::AccountEntryList, *},
    event_handlers::request_handlers::{create_response_listener, create_response_process_cb},
    pages::home_page::HomePage,
    pages::login_page::LoginPage,
    store::EXTENSION_PORT,
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
    let passphrase_state = use_state(|| None);
    let passphrase_state_cloned = passphrase_state.clone();
    let on_message_cb = create_response_listener(EXTENSION_PORT.clone());
    let port = EXTENSION_PORT.clone();
    port.on_message()
        .add_listener(on_message_cb.as_ref().clone());
    on_message_cb.forget();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                log!("inside spawned executor");
                // chrome
                //     .storage()
                //     .session()
                //     .set_string_item("passphrase".to_owned(), "abcd".to_owned())
                //     .await;
                if let Ok(passphrase) = chrome
                    .storage()
                    .session()
                    .get_string_value("passphrase")
                    .await
                {
                    if let Some(passphrase) = passphrase {
                        passphrase_state_cloned.set(Some(passphrase));

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
                            <JsValue as JsValueSerdeExt>::from_serde(&get_password_request)
                                .unwrap(),
                        );
                        port.post_message(
                            <JsValue as JsValueSerdeExt>::from_serde(&get_username_request)
                                .unwrap(),
                        );
                    } else {
                        log!("passphrase is not set");
                    }
                }
            });
        },
        (),
    );

    html! {
        <HomePage passphrase={passphrase_state.deref().clone()}/>
    }
}
