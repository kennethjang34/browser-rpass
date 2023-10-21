use crate::{
    api::extension_api::{create_account, fetch_accounts, logout},
    pages::login_page::LoginPage,
    store::PopupStore,
    Account,
};
use log::log;
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
    log!("render home page");
    let path = use_selector(|state: &PopupStore| state.path.clone());
    let loading = use_selector(|state: &PopupStore| state.page_loading.clone());
    let verified = use_selector(|state: &PopupStore| state.verified);
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
        <div>
        if *loading {
            <p>{"loading..."}</p>
        }else{
            if *verified{
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
                <button onclick={on_logout_click}>{ "Logout" }</button>
                <form  onsubmit={on_create_submit}>
                <label for="new-username">{"new username:"}</label>
                <input type="text" id="new-username" name="create-username" ref={username_input_ref}/><br/>
                <label for="new-password">{"new password:"}</label>
                <input type="password" id="new-password" name="create-password" ref={password_input_ref}/><br/>
                <button>{ "Create" }</button>
                </form>
            </div>
            }else{
                <p>{"need to login!"}</p>
                    <LoginPage />
            }
        }
        </div>
    }
}
