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
    let bottom_tooltip_span = move |text| -> Html {
        html! {
            <div
                style="min-width: fit-content; left: 0.3rem;padding: 0.3rem;bottom: 1.3rem;border-width: thin;border-style: dashed;"
                class="transition-opacity px-1  text-sm  border-gray-800 dark:border-gray-400 rounded-md fixed left-0 bottom-0 translate-y-full opacity-0 m-4 mx-auto text-black dark:text-white
peer-hover:opacity-100 
                "
                >
                {text}
            </div>
        }
    };
    let opened_eye_icon = html! { <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 14">
    <g stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
        <path d="M10 10a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/>
        <path d="M10 13c4.97 0 9-2.686 9-6s-4.03-6-9-6-9 2.686-9 6 4.03 6 9 6Z"/>
        </g>
        </svg> };
    let closed_eye_icon = html! {
        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 18">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1.933 10.909A4.357 4.357 0 0 1 1 9c0-1 4-6 9-6m7.6 3.8A5.068 5.068 0 0 1 19 9c0 1-3 6-9 6-.314 0-.62-.014-.918-.04M2 17 18 1m-5 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"/>
        </svg>
    };
    let password_cell = |revealed: bool| -> Html {
        let (eye_tooltip_text, password_text, eye_icon) = if revealed {
            (
                "click to hide password",
                password.clone().unwrap_or_default(),
                closed_eye_icon,
            )
        } else {
            (
                "click to reveal password",
                "************".to_string(),
                opened_eye_icon,
            )
        };
        html! {
            <>
                <div class="account-password-cell">
                    // <div class="group overflow-x-auto cursor-copy" style="justify-self:center;">
                    <div class="overflow-x-auto cursor-copy" style="justify-self:center;">
                        <span class="peer" onclick={copy_pw.clone()}>
                        // <span onclick={copy_pw.clone()}>
                            {password_text}
                        </span>
                    {bottom_tooltip_span("click to copy password")}
                    </div>
                </div>
                // <div class="group">
                <div>
                    // <span onclick={on_reveal} class="cursor-pointer" style="transform: translateY(-50%);">
                    <span onclick={on_reveal} class="cursor-pointer peer" style="transform: translateY(-50%);">
                        {eye_icon}
                    </span>
                    {bottom_tooltip_span(eye_tooltip_text)}
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
                                    {bottom_tooltip_span("click to copy domain")}
                            </div>
                            <div class="account-username">
                                <span class="peer cursor-copy select-all"  onclick={copy_username.clone()} >
                                {
                                    username.clone()
                                 }
                                </span>
                                    {bottom_tooltip_span("click to copy username")}
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
