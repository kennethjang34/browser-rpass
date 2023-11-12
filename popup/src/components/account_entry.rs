use std::rc::Rc;

use crate::Account;
use browser_rpass::util::*;
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
    html! {
        <>
                    <td scope="row" class="px-3 py-2 font-medium bold text-gray-900 whitespace-nowrap dark:text-white">
                            <div>
                                <div style="max-width: fit-content;" class="group text-xs">
                                    <span class="cursor-pointer text-gray-500 text-xs font-normal select-all" onclick={copy_domain.clone()}>
                                        {domain.as_ref().unwrap_or(&"".to_string())}
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto">
                                        {"click to copy domain"}
                                    </span>
                                </div>
                                <div style="max-width: fit-content;" class="group">
                                    <span class="cursor-pointer select-all"  onclick={copy_username.clone()} >
                                    {
                                        username.clone()
                                     }
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto">
                                        {"click to copy username"}
                                    </span>
                                </div>
                            </div>
                    </td>
                    <td class="px-3 py-2 font-medium">
                            <div>
                            if *reveal_password {
                                <div style="max-width: fit-content;" class="group">
                                    <span class="cursor-pointer" onclick={copy_pw.clone()}>
                                        {password.clone()}
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to copy password"}
                                    </span>
                                </div>
                                <div style="max-width: fit-content;" class="group">
                                    <button onclick={on_reveal}>{"Hide"}</button>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to hide password"}
                                    </span>
                                </div>

                            } else{
                                <div style="max-width: fit-content;" class="group">
                                    <span class="cursor-pointer" onclick={copy_pw.clone()}>
                                        {"**********"}
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to copy password"}
                                    </span>
                                </div>
                                <div style="max-width: fit-content;" class="group">
                                    <button class="cursor-pointer" onclick={on_reveal}>{"Show"}</button>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to reveal password"}
                                    </span>
                                </div>
                            }
                        </div>
                    </td>
                </>
    }
}
