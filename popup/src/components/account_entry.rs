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
                    <td>
                        <div >
                                <p>
                                    {domain.clone()}
                                </p>
                            </div>
                        <button onclick={copy_domain.clone()}>{"copy domain"}</button>
                    </td>
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
