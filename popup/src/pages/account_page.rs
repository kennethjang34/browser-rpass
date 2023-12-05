use crate::components::AccountEntryList;
use crate::components::CloseButton;
use crate::components::CreateAccountPopup;
use crate::components::SearchInput;
use crate::store::{DataAction, PopupStore, StoreDataStatus};
use browser_rpass::types::Account;
#[allow(unused_imports)]
use log::*;
use std::collections::BTreeMap;
use std::rc::Rc;
use sublime_fuzzy::best_match;
use wasm_bindgen::JsCast;
use yew;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub user_id: Option<String>,
    pub path: Option<String>,
}

#[function_component(AccountPage)]
pub fn account_page(props: &Props) -> Html {
    trace!("account page");
    let search_string = use_state(|| String::new());
    let path = props.path.clone();
    let account_selector = use_selector_with_deps(
        |state: &PopupStore, (search_string, path)| {
            trace!("inside selector function");
            let accounts = state.data.accounts.clone();
            let accounts = {
                if search_string.is_empty() {
                    let mut result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                    let mut non_matched: Vec<Rc<Account>> = vec![];
                    accounts.borrow().iter().cloned().for_each(|account| {
                        let domain = account.domain.as_ref();
                        let path = path.clone().unwrap_or("".to_owned());
                        let m_res = best_match(&path, domain.unwrap_or(&String::new()));
                        if let Some(m_res) = m_res {
                            let score = m_res.score();
                            result
                                .entry(score)
                                .and_modify(|ls| ls.push(account.clone()))
                                .or_insert(vec![account.clone()]);
                        } else {
                            non_matched.push(account);
                        }
                    });
                    let mut result_vec: Vec<Rc<Account>> = vec![];
                    for vac in result.values() {
                        for v in vac {
                            result_vec.push((*v).clone());
                        }
                    }
                    for v in non_matched {
                        result_vec.push(v.clone());
                    }
                    result_vec
                } else {
                    let mut search_result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                    let account_data: Vec<Rc<Account>> =
                        accounts.borrow().iter().cloned().collect();
                    for account in account_data {
                        let account_username = &account.username;
                        let result = best_match(&search_string, account_username);
                        if let Some(result) = result {
                            let score = result.score();
                            // following is to avoid cloning
                            search_result
                                .entry(score)
                                .and_modify(|ls| ls.push(account.clone()))
                                .or_insert_with(|| vec![account.clone()]);
                        }
                    }
                    let mut result_vec: Vec<Rc<Account>> = vec![];
                    for vac in search_result.into_values() {
                        for v in vac {
                            result_vec.push(v);
                        }
                    }
                    result_vec
                }
            };
            accounts
        },
        (search_string.clone(), path.clone()),
    );
    let on_search = Callback::from({
        let search_string = search_string.clone();
        move |event: InputEvent| {
            event.prevent_default();
            search_string.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        }
    });
    let show_create_account_popup = use_state(|| false);
    let on_create_account = {
        let show_create_account_popup = show_create_account_popup.clone();
        Callback::from(move |_: MouseEvent| {
            let value = !*show_create_account_popup;
            show_create_account_popup.set(value);
        })
    };
    let store_dispatch = Dispatch::<PopupStore>::new();
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());

    let close_create_account_popup = {
        let show_create_account_popup = show_create_account_popup.clone();
        Callback::from({
            let store_dispatch = store_dispatch.clone();
            move |_: MouseEvent| {
                show_create_account_popup.set(false);
                store_dispatch.apply(DataAction::Idle);
            }
        })
    };
    let close_error = {
        let dispatch = store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };
    let table_header_element = move |text: &str| -> Html {
        html! {
        <th scope="col">
            {
            text
            }</th>
        }
    };
    let table_headers = ["username", "password", "note", "", ""];
    html! {
            <>
                <div class="relative overflow-hidden shadow-md sm:rounded-lg w-full h-full">
                <div class="w-full top-2.5" style="border-bottom:outset; height: 80%;">
                <SearchInput onchange={on_search} value={(*search_string).clone()}/>
                    if *store_status==StoreDataStatus::DeletionFailed{
                        <div id="toast-danger" class="flex absolute right-0 items-center max-w-xs p-2 my-4 text-gray-500 bg-white rounded-lg shadow dark:text-gray-400 dark:bg-gray-800 z-10 top-5" role="alert">
                            <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 text-red-500 bg-red-100 rounded-lg dark:bg-red-800 dark:text-red-200 ">
                            <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/>
                            </svg>
                            <span class="sr-only">{"Error icon"}</span>
                            </div>
                            <div class="ms-3 text-sm font-normal">{"Deletion Failed"}</div>
                            <CloseButton onclick={close_error}
                                class="-my-1.5 "
                            />
                        </div>
                    }
                    <table class="dark:text-gray-400 relative rtl:text-right text-gray-500 text-left text-sm w-full top-3" border="1">
                            <colgroup>
                            <col  span="1" />
                            <col  span="1" />
                            <col  span="1"/>
                            <col  span="1"/>
                            <col  span="1"/>
                            </colgroup>
                        <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400 w-full">
                            <tr>
                                {
                                    table_headers.iter().map(|header| table_header_element(header)).collect::<Html>()
                                }
                            </tr>
                        </thead>
                        <tbody>
                            <AccountEntryList accounts={account_selector}/>
                        </tbody>
                    </table>
                    </div>
                    <button  class="primary-btn block  my-4" type="button" onclick={on_create_account}>
                    {"Create Account"}
    </button>
                    if *show_create_account_popup{
                        <div class="fullscreen-container">
                            <CreateAccountPopup domain={props.path.clone()} handle_close={close_create_account_popup}/>
                        </div>
                    }
                </div>
            </>
        }
}
