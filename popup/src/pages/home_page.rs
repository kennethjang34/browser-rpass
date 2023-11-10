use crate::{
    api::extension_api::{create_account, fetch_accounts, logout},
    pages::login_page::LoginPage,
    store::PopupStore,
    Account,
};
use log::*;
use std::{collections::BTreeMap, rc::Rc};
use sublime_fuzzy::best_match;

use crate::components::account_entry_list::AccountEntryList;
use web_sys::HtmlInputElement;
use yew;
use yew::prelude::*;
use yewdux::{self};
use yewdux::{mrc::Mrc, prelude::use_selector};

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(HomePage)]
pub fn home_page(_props: &Props) -> Html {
    trace!("render home page");
    let path = use_selector(|state: &PopupStore| state.path.clone());
    let loading = use_selector(|state: &PopupStore| state.page_loading.clone());
    let verified = use_selector(|state: &PopupStore| state.verified);
    let user_id = use_selector(|state: &PopupStore| state.user_id.clone());
    let account_selector = use_selector(|state: &PopupStore| state.data.accounts.clone());
    let accounts = use_state(|| Rc::new(Vec::<Rc<Account>>::new()));
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
    let search_string = use_state(|| String::new());
    let search_input_ref = NodeRef::default();
    use_effect_with_deps(
        {
            let _path = path.clone();
            move |verified: &Rc<bool>| {
                if **verified {
                    fetch_accounts(None);
                }
            }
        },
        verified.clone(),
    );
    use_effect_with_deps(
        {
            let accounts = accounts.clone();
            let verified = verified.clone();
            move |(path, account_selector): &(Rc<Option<String>>, Rc<Mrc<Vec<Rc<Account>>>>)| {
                if *verified {
                    let account_state = account_selector.clone();
                    let mut result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                    let mut non_matched: Vec<Rc<Account>> = vec![];
                    account_state.borrow().iter().cloned().for_each(|account| {
                        let domain = account.domain.as_ref();
                        let path = (**path).clone().unwrap_or("".to_owned());
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
                    accounts.set(Rc::new(result_vec));
                }
            }
        },
        (path.clone(), account_selector.clone()),
    );

    use_effect_with_deps(
        {
            let accounts = accounts.clone();
            let path = path.clone();
            move |search_string: &UseStateHandle<String>| {
                if search_string.is_empty() {
                    let mut result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                    let mut non_matched: Vec<Rc<Account>> = vec![];
                    account_selector
                        .borrow()
                        .iter()
                        .cloned()
                        .for_each(|account| {
                            let domain = account.domain.as_ref();
                            let path = (*path).clone().unwrap_or("".to_owned());
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
                    accounts.set(Rc::new(result_vec));
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
                    accounts.set(Rc::new(result_vec));
                }
            }
        },
        search_string.clone(),
    );
    let on_search = Callback::from({
        let search_input_ref = search_input_ref.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let search_input = search_input_ref.cast::<HtmlInputElement>().unwrap();
            search_string.set(search_input.value());
        }
    });
    let on_create_submit = Callback::from({
        let path = (*path).clone();
        let password_input_ref = password_input_ref.clone();
        let username_input_ref = username_input_ref.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let password_input = password_input_ref.cast::<HtmlInputElement>().unwrap();
            let username_input = username_input_ref.cast::<HtmlInputElement>().unwrap();
            let path = path.clone();
            create_account(
                path.unwrap_or_default(),
                username_input.value(),
                password_input.value(),
            );
            username_input.set_value("");
            password_input.set_value("");
        }
    });
    let on_logout_click = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        logout();
    });
    html! {
                        <div id="authentication-modal" tabindex="-1" aria-hidden="true" class="w-96 h-96 top-0 left-0 right-0 z-50 overflow-x-hidden overflow-y-auto md:inset-0"
                >
                   <div class="w-full h-full">
                      <div class="relative w-full h-full max-w-md max-h-full">
                         <div class="relative bg-white rounded-lg shadow dark:bg-gray-700 w-full h-full">
                            <button type="button" class="absolute top-3 right-2.5 text-gray-400 bg-transparent hover:bg-gray-200 hover:text-gray-900 rounded-lg text-sm w-6 h-6 ml-auto inline-flex justify-center items-center dark:hover:bg-gray-600 dark:hover:text-white" >
                               <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                                  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                               </svg>
                               <span class="sr-only">{"close modal"}</span>
                            </button>
                            <div class="px-3 py-1.5 lg:px-8">

            if *loading {
                    <div class="flex items-center justify-center w-full h-full border border-gray-200 rounded-lg bg-gray-50 dark:bg-gray-800 dark:border-gray-700">
        <div role="status">
            <svg aria-hidden="true" class="w-full h-full mr-2 text-gray-200 animate-spin dark:text-gray-600 fill-blue-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="currentColor"/><path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/></svg>
            <span class="sr-only">{"Loading..."}</span>
        </div>
    </div>
            }else{
                if *verified{
                if user_id.is_some(){
                    <p>{format!("logged in as {}", (*user_id).as_ref().unwrap())}</p>
                }
                <form  onsubmit={on_search}>
                <label for="account-search">{"Search for account:"}</label><br/>
                <input type="search" id="account-search" name="account-search" ref={search_input_ref}/>
            <button >{ "Search" }</button>
                </form>
            <table class="table table-bordered">
                <thead>
                  <tr>
                    <th>{ "Domain" }</th>
                    <th>{ "Username" }</th>
                    <th>{ "Password" }</th>
                  </tr>
                </thead>
                <tbody>
                <AccountEntryList
                accounts={(*accounts).clone()}
                />
                </tbody>
            </table>
            <div>
                <form  onsubmit={on_create_submit}>
                <label for="new-username">{"new username:"}</label>
                <input type="text" id="new-username" name="create-username" ref={username_input_ref}/><br/>
                <label for="new-password">{"new password:"}</label>
                <input type="password" id="new-password" name="create-password" ref={password_input_ref}/><br/>
                <button>{ "Create" }</button>
                </form>
            </div>
                    <button type="button" class="text-red-700 hover:text-white border border-red-700 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center mr-2 mb-2 dark:border-red-500 dark:text-red-500 dark:hover:text-white dark:hover:bg-red-600 dark:focus:ring-red-900" onclick={on_logout_click}>{"logout"}</button>
                }
                else{
                        <LoginPage />
                }
            }
            </div>
            </div>
            </div>
            </div>
            </div>
    }
}
