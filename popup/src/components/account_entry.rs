use crate::components::*;
use crate::Account;
use browser_rpass::js_binding::clipboard_copy;
use std::rc::Rc;
use wasm_bindgen_futures;
use yew;

use yew::prelude::*;

#[derive(Debug, PartialEq, Properties)]
pub struct AccountEntryProps {
    pub id: usize,
    pub account: Rc<Account>,
}

#[function_component(AccountEntry)]
pub fn account_entry_component(props: &AccountEntryProps) -> Html {
    let account = props.account.clone();
    let password = &account.password;
    let reveal_password = use_state(|| false);
    let on_reveal = {
        let reveal_password = reveal_password.clone();
        Callback::from(move |_: MouseEvent| {
            let value = !*reveal_password;
            reveal_password.set(value);
        })
    };
    let username = &account.username;
    let domain = &account.domain;
    let note = &account.note;
    let copy_domain = {
        let domain = domain.clone();
        Callback::from({
            move |_: MouseEvent| {
                let domain = domain.clone();
                if let Some(domain) = domain {
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = clipboard_copy(&domain).await;
                    });
                }
            }
        })
    };
    let copy_username = {
        let username = username.clone();
        Callback::from({
            move |_: MouseEvent| {
                let username = username.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = clipboard_copy(&username).await;
                });
            }
        })
    };

    let copy_pw = {
        let password = password.clone();
        Callback::from({
            move |_: MouseEvent| {
                if let Some(password) = password.as_ref() {
                    let password = password.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        _ = clipboard_copy(&password).await;
                    });
                }
            }
        })
    };
    let password_cell = |revealed: bool| -> Html {
        let (eye_tooltip_text, password_text, eye_icon) = if revealed {
            (
                "click to hide password",
                password.clone().unwrap_or_default(),
                html! {<ClosedEyeIcon/>},
            )
        } else {
            (
                "click to reveal password",
                "************".to_string(),
                html! {<OpenEyeIcon/>},
            )
        };
        html! {
            <>
                <div class="account-password-cell">
                    <div class="overflow-x-auto" style="justify-self:center;">
                        <span class="peer cursor-copy" onclick={copy_pw.clone()}>
                            {password_text}
                        </span>
                            <Tooltip text={"click to copy password".to_string()} class="bottom-tooltip"/>
                    </div>
                </div>
                <div>
                    <span onclick={on_reveal} class="cursor-pointer peer" style="transform: translateY(-50%);">
                        {eye_icon}
                    </span>
                    <Tooltip text={eye_tooltip_text.to_string()} class="bottom-tooltip"/>
                </div>
            </>
        }
    };

    html! {
        <>
                    <th scope="row" class="font-medium bold text-gray-900 whitespace-nowrap dark:text-white">
                            <div class="account-username-cell">
                            <div class="account-domain">
                                <span class="peer cursor-copy text-gray-500" onclick={copy_domain.clone()}>
                                    {domain.as_ref().unwrap_or(&"".to_string())}
                                </span>
                                    <Tooltip text={"click to copy domain".to_string()} class="bottom-tooltip"/>
                            </div>
                            <div class="account-username">
                                <span class="peer cursor-copy select-all"  onclick={copy_username.clone()} >
                                {
                                    username.clone()
                                 }
                                </span>
                                    <Tooltip text={"click to copy username".to_string()} class="bottom-tooltip"/>
                            </div>
                        </div>
                    </th>
                    <td scope="row" class="py-2 font-medium">
                        <div class="password">
                        {password_cell(*reveal_password)}
                        </div>
                    </td>
                    <td scope="row" class="font-medium text-gray-900 whitespace-nowrap dark:text-white">
                        <div>
                            <div class="overflow-x-auto text-center">
                                <span class="peer select-all">
                                {
                                    note.clone()
                                 }
                                </span>
                            </div>
                        </div>
                    </td>
                </>
    }
}
