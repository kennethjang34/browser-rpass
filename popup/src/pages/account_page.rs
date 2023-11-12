use crate::components::account_entry_list::AccountEntryList;
use crate::components::create_account_popup::CreateAccountPopup;
use crate::store::PopupStore;
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
    let account_selector = use_selector(|state: &PopupStore| state.data.accounts.clone());
    let search_string = use_state(|| String::new());
    let search_input_ref = NodeRef::default();
    let path = props.path.clone();
    let accounts = {
        if search_string.is_empty() {
            let mut result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
            let mut non_matched: Vec<Rc<Account>> = vec![];
            account_selector
                .borrow()
                .iter()
                .cloned()
                .for_each(|account| {
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
                account_selector.borrow().iter().cloned().collect();
            for account in account_data {
                let account_id = &account.id;
                let result = best_match(&search_string, account_id);
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
    let on_search = Callback::from({
        let search_input_ref = search_input_ref.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let search_input = search_input_ref.cast::<HtmlInputElement>().unwrap();
            search_string.set(search_input.value());
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
    let close_create_account_popup = {
        let show_create_account_popup = show_create_account_popup.clone();
        Callback::from(move |_: MouseEvent| {
            show_create_account_popup.set(false);
        })
    };
    html! {
            <>
                <div class="relative overflow-hidden shadow-md sm:rounded-lg w-full h-full">
                <div class="w-full" style="min-height:350px; border-bottom:outset;">
                    <label for="table-search" class="sr-only">{"Search"}</label>
                    <form class="relative mt-1 mb-3 top-14" onsubmit={on_search}>
                        <div class="absolute inset-y-0 rtl:inset-r-0 start-0 flex items-center ps-3 pointer-events-none">
                            <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"/>
                            </svg>
                        </div>
                        <input type="text" id="table-search" class="block pt-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg w-80 bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search for items"
                         />
                    </form>
                    <table class="dark:text-gray-400 mt-14 relative rtl:text-right text-gray-500 text-left text-sm top-3 w-full">
                        <thead class="text-xs text-gray-700 uppercase bg-gray-50 dark:bg-gray-700 dark:text-gray-400">
                            <tr>
                                <th scope="col" class="px-3 py-2">
                                {
                                    "Username"
                                }
                                </th>
                                <th scope="col" class="px-3 py-2">
                                {
                                    "Password"
                                }
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <AccountEntryList accounts={Rc::new(accounts)}/>
                        </tbody>
                    </table>
                    </div>
                    <button  class="bg-white block dark:bg-gray-800 dark:focus:ring-gray-800 dark:hover:bg-gray-600 relative focus:outline-none focus:ring-4 focus:ring-gray-50 font-medium hover:bg-gray-50 px-5 py-2.5 rounded-lg text-center text-sm text-blue-600 dark:text-blue-500 my-4" type="button" onclick={on_create_account}>
                    {"Create Account"}
    </button>
                    if *show_create_account_popup{
                        <CreateAccountPopup domain={props.path.clone()} handle_close={close_create_account_popup}/>
                    }
                </div>
            </>
        }
}
