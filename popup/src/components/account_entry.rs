use wasm_bindgen_futures::spawn_local;
// use web_sys::Navigator;
use browser_rpass::log;
use browser_rpass::util::*;

use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{api::types::Account, store::Store};

#[derive(Debug, PartialEq, Properties)]
pub struct AccountEntryProps {
    pub id: usize,
    pub account: Account,
}

#[function_component(AccountEntry)]
pub fn account_entry_component(props: &AccountEntryProps) -> Html {
    let (store, dispatch) = use_store::<Store>();
    let account = props.account.clone();
    let email = account.email.clone();
    let password = account.password.clone();
    let reveal_password = use_state(|| false);
    let on_reveal = {
        let reveal_password = reveal_password.clone();
        Callback::from(move |_: MouseEvent| {
            let value = !*reveal_password;
            reveal_password.set(value);
        })
    };
    let username = {
        if let Some(username) = account.username.as_ref() {
            username.clone()
        } else {
            account.email.clone()
        }
    };

    let copy_username = {
        let store_dispatch = dispatch.clone();
        let username = username.clone();
        Callback::from({
            move |_: MouseEvent| {
                let window = web_sys::window().expect("Missing Window");
                let navigator = window.navigator();
                let username = username.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    clipboard_copy(&username).await;
                });
            }
        })
    };

    let copy_pw = {
        let store_dispatch = dispatch.clone();
        let password = password.clone();
        Callback::from({
            move |_: MouseEvent| {
                let window = web_sys::window().expect("Missing Window");
                let navigator = window.navigator();
                if let Some(password) = password.as_ref() {
                    let password = password.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        clipboard_copy(&password).await;
                    });
                }
            }
        })
    };
    html! {
        <>
                    <td class="email">
                        <div class="pressable" onclick={copy_username.clone()}>
                        <p class="pressable" >
                        {username.clone()}
        </p>

            </div>
                        <button onclick={copy_username.clone()}>{"copy username"}</button>
                    </td>
                    <td>
                        <div class="pressable" onclick={copy_pw.clone()}>
                    if *reveal_password {
                            <p>
                        {password.clone()}
                        </p>
                        <button onclick={copy_pw.clone()}>{"copy password"}</button>
                        <button onclick={on_reveal}>{"Hide"}</button>
                    } else{
                        <p onclick={copy_pw.clone()}>
                        {"**********"}
                        </p>
                        <button onclick={copy_pw.clone()}>{"copy password"}</button>
                        <button onclick={on_reveal}>{"Show"}</button>
                    }
                    </div>
                    </td>
                    </>
    }
}
