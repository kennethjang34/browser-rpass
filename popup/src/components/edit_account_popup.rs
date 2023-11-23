use std::rc::Rc;

use crate::{
    api::extension_api::edit_account,
    store::{DataAction, PopupStore, StoreData, StoreDataStatus},
};
use browser_rpass::types::Account;
#[allow(unused_imports)]
use log::*;
use wasm_bindgen::JsCast;
use yew;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub account: Rc<Account>,
    pub handle_close: Callback<MouseEvent>,
}

#[function_component(EditAccountPopup)]
pub fn edit_account_popup(props: &Props) -> Html {
    let reveal_password = use_state(|| false);
    let account = props.account.clone();
    let password_input = use_state(|| account.password.clone().unwrap());
    let username_input = use_state(|| account.username.clone());
    let domain_input = use_state(|| account.domain.clone().unwrap());
    let on_edit_submit = Callback::from({
        let account = account.clone();
        let password_input = password_input.clone();
        let username_input = username_input.clone();
        let domain_input = domain_input.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let domain = if (*domain_input).is_empty() {
                None
            } else {
                Some((*domain_input).clone())
            };
            let username = if (*username_input).is_empty() {
                None
            } else {
                Some((*username_input).clone())
            };
            let password = if (*password_input).is_empty() {
                None
            } else {
                Some((*password_input).clone())
            };
            edit_account(account.id.clone(), domain, username, password);
        }
    });
    let on_reveal = {
        let reveal_password = reveal_password.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let value = !*reveal_password;
            reveal_password.set(value);
        })
    };
    let on_password_input = {
        let password_input = password_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            password_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let on_username_input = {
        let username_input = username_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            username_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let on_domain_input = {
        let domain_input = domain_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            domain_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    use_effect_with_deps(
        {
            let popup_store_dispatch = popup_store_dispatch.clone();
            move |(store_status, handle_close): &(Rc<StoreDataStatus>, Callback<MouseEvent>)| {
                if **store_status == StoreDataStatus::EditionSuccess {
                    handle_close.emit(MouseEvent::new("click").unwrap());
                    popup_store_dispatch.apply(DataAction::Idle);
                }
            }
        },
        (store_status.clone(), props.handle_close.clone()),
    );
    let close_error = {
        let dispatch = popup_store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };
    html! {
        <div id="edit-account-popup" tabindex="-1" aria-hidden="true" class="overflow-y-auto overflow-x-hidden shadow-lg fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full">
            <div class="relative p-4 w-full max-h-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900">
                    <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {"Edit Account"}
                        </h3>
                if *store_status == StoreDataStatus::EditionFailed {
                    <div id="toast-danger" class="flex absolute right-0 items-center w-full max-w-xs p-4 mr-5 my-4 text-gray-500 bg-white rounded-lg shadow dark:text-gray-400 dark:bg-gray-800" role="alert">
                        <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 text-red-500 bg-red-100 rounded-lg dark:bg-red-800 dark:text-red-200">
                        <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/>
                        </svg>
                        <span class="sr-only">{"Error icon"}</span>
                        </div>
                        <div class="ms-3 text-sm font-normal">{"Edition Failed"}</div>
                        <button type="button" onclick={
                            close_error
                        } class="ms-auto -mx-1.5 -my-1.5 bg-white text-gray-400 hover:text-gray-900 rounded-lg focus:ring-2 focus:ring-gray-300 p-1.5 hover:bg-gray-100 inline-flex items-center justify-center h-8 w-8 dark:text-gray-500 dark:hover:text-white dark:bg-gray-800 dark:hover:bg-gray-700" data-dismiss-target="#toast-danger" aria-label="Close">
                    <span class="sr-only">{"Close"}</span>
                    <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                    <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                    </svg>
                    </button>
                    </div>
                }
                        <button type="button" class="text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-8 h-8 ms-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white" onclick={&props.handle_close}>
                            <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                            </svg>
                            <span class="sr-only">{"Close"}</span>
                        </button>
                    </div>
                    <form onsubmit={on_edit_submit} class="p-4 md:p-5">
                        <div class="grid gap-4 mb-4 grid-cols-2">
                            <div class="col-span-2">
                                <label for="user-id" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{"User ID"}</label>
                                <input type="text" name="user-id" id="user-id" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="User ID" required={true} value={(*username_input).clone()} oninput={on_username_input.clone()}/>
                            </div>
                            if !*reveal_password {
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="password" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Password" }</label>
                                    <div class="relative">
                                    <input type="password" name="password" id="password" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="Password" required={true} value={(*password_input).clone()} oninput={on_password_input} />
                                    <span onclick={on_reveal} class="absolute cursor-pointer right-2 top-1/2" style="transform: translateY(-50%);">
                                        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 14">
                                            <g stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
                                              <path d="M10 10a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/>
                                              <path d="M10 13c4.97 0 9-2.686 9-6s-4.03-6-9-6-9 2.686-9 6 4.03 6 9 6Z"/>
                                            </g>
                                        </svg>
                                    </span>
                                    </div>
                                </div>
                            }else{
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="password" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Password" }</label>
                                    <div class="relative">
                                    <input type="text" name="password" id="password" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="Password" required={true} value={(*password_input).clone()} oninput={on_password_input}/>
                                    <span onclick={on_reveal} class="absolute cursor-pointer right-2 top-1/2" style="transform: translateY(-50%);">
                                        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 18">
                                            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1.933 10.909A4.357 4.357 0 0 1 1 9c0-1 4-6 9-6m7.6 3.8A5.068 5.068 0 0 1 19 9c0 1-3 6-9 6-.314 0-.62-.014-.918-.04M2 17 18 1m-5 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"/>
                                        </svg>
                                    </span>
                                    </div>
                                </div>
                            }
                            <div class="col-span-2 sm:col-span-1">
                                <label for="url" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "URL" }</label>
                                <input type="text" name="url" id="url" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="URL" required={true} value={(*domain_input).clone()}  oninput={on_domain_input.clone()}/>
                            </div>
                        </div>
                        <button type="submit" class="text-white inline-flex items-center bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                            <svg class="me-1 -ms-1 w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd"></path></svg>
                            {"Update Account"}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
