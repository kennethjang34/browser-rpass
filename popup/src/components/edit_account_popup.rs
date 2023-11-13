use std::rc::Rc;

use crate::api::extension_api::edit_account;
use browser_rpass::types::Account;
#[allow(unused_imports)]
use log::*;
use yew;

use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub account: Rc<Account>,
    pub handle_close: Callback<MouseEvent>,
}

#[function_component(EditAccountPopup)]
pub fn edit_account_popup(props: &Props) -> Html {
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
    let domain_input_ref = NodeRef::default();
    let account = props.account.clone();
    let on_edit_submit = Callback::from({
        let password_input_ref = password_input_ref.clone();
        let username_input_ref = username_input_ref.clone();
        let domain_input_ref = domain_input_ref.clone();
        let account = account.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let password_input = password_input_ref.cast::<HtmlInputElement>().unwrap();
            let username_input = username_input_ref.cast::<HtmlInputElement>().unwrap();
            let domain_input = domain_input_ref.cast::<HtmlInputElement>().unwrap();
            let domain = if domain_input.value().is_empty() {
                None
            } else {
                Some(domain_input.value())
            };
            let username = if username_input.value().is_empty() {
                None
            } else {
                Some(username_input.value())
            };
            let password = if password_input.value().is_empty() {
                None
            } else {
                Some(password_input.value())
            };
            edit_account(account.id.clone(), domain, username, password);
            username_input.set_value("");
            password_input.set_value("");
        }
    });
    html! {

        <div id="edit-account-popup" tabindex="-1" aria-hidden="true" class="overflow-y-auto overflow-x-hidden shadow-lg fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0 h-[calc(100%-1rem)] max-h-full">
            <div class="relative p-4 w-full max-h-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900">
                    <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {"Edit Account"}
                        </h3>
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
                                <input type="text" name="user-id" id="user-id" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="User ID" required={true} value={account.username.clone()} ref={username_input_ref}/>
                            </div>
                            <div class="col-span-2 sm:col-span-1">
                                <label for="price" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Password" }</label>
                                <input type="password" name="password" id="password" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="Password" required={true} value={account.password.clone().unwrap()} ref={password_input_ref}/>
                            </div>
                            <div class="col-span-2 sm:col-span-1">
                                <label for="url" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "URL" }</label>
                                <input type="text" name="url" id="url" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-primary-600 focus:border-primary-600 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white dark:focus:ring-primary-500 dark:focus:border-primary-500" placeholder="URL" required={true} value={account.domain.clone().unwrap_or_default()} ref={domain_input_ref}/>
                            </div>
                        </div>
                        <button type="submit" class="text-white inline-flex items-center bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                            <svg class="me-1 -ms-1 w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd"></path></svg>
                            {"Edit"}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}