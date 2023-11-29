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
    html! {
        <>
                    <td scope="row" class="px-3 py-2 font-medium bold text-gray-900 whitespace-nowrap dark:text-white">
                        <div>
                            <div style="max-width: fit-content;" class="group text-xs overflow-x-hidden">
                                <span class="cursor-copy text-gray-500 text-xs font-normal select-all" onclick={copy_domain.clone()}>
                                    {domain.as_ref().unwrap_or(&"".to_string())}
                                </span>
                                <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto">
                                    {"click to copy domain"}
                                </span>
                            </div>
                            <div style="width: 12rem;" class="group overflow-x-auto">
                                <span class="cursor-copy select-all"  onclick={copy_username.clone()} >
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
                    <td class="px-3 py-2 font-medium" style="min-width: 9rem;">
                            <div class="relative">
                            if *reveal_password {
                                <div style="width: 6rem;">
                                    <div class="group overflow-x-auto cursor-copy" style="max-width: fit-content;">
                                    <span onclick={copy_pw.clone()}>
                                        {password.clone()}
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to copy password"}
                                    </span>
                                    </div>
                                    </div>
                                <div class="group">
                                    <span onclick={on_reveal} class="absolute cursor-pointer top-1/2 ms-4" style="transform: translateY(-50%); left:3.5rem;">
                                        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 18">
                                            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1.933 10.909A4.357 4.357 0 0 1 1 9c0-1 4-6 9-6m7.6 3.8A5.068 5.068 0 0 1 19 9c0 1-3 6-9 6-.314 0-.62-.014-.918-.04M2 17 18 1m-5 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"/>
                                        </svg>
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to hide password"}
                                    </span>
                                </div>
                            } else
                            {
                                <div style="width: 6rem;">
                                    <div class="group overflow-x-auto cursor-copy" style="max-width: fit-content;">
                                    <span onclick={copy_pw.clone()}>
                                        {"**********"}
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to copy password"}
                                    </span>
                                    </div>
                                    </div>
                                    <div class="group">
                                    <span onclick={on_reveal} class="absolute cursor-pointer top-1/2 ms-4" style="transform: translateY(-50%); left:3.5rem;">
                                        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 14">
                                            <g stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
                                              <path d="M10 10a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/>
                                              <path d="M10 13c4.97 0 9-2.686 9-6s-4.03-6-9-6-9 2.686-9 6 4.03 6 9 6Z"/>
                                            </g>
                                        </svg>
                                    </span>
                                    <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto dark:text-white">
                                        {"click to reveal password"}
                                    </span>
                                    </div>
                            }
                        </div>
                    </td>
                    <td scope="row" class="px-3 py-2 font-medium bold text-gray-900 whitespace-nowrap dark:text-white">
                        <div>
                            <div style="width: 5rem;" class="group overflow-x-auto text-center">
                                <span class="select-all">
                                // <span class="cursor-copy select-all"  onclick={copy_username.clone()} >
                                {
                                    note.clone()
                                 }
                                </span>
                                // <span class="group-hover:opacity-100 transition-opacity bg-gray-800 px-1 text-sm text-gray-100 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto">
                                    // {"click to copy username"}
                                // </span>
                            </div>
                        </div>
                    </td>
                </>
    }
}
