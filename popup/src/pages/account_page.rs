use crate::api::extension_api::{create_account, fetch_accounts, login};
use crate::components::account_entry_list::AccountEntryList;
// use crate::api::user_api::api_login_user;
use crate::components::{form_input::FormInput, loading_button::LoadingButton};
use browser_rpass::types::Account;
use log::*;
use serde_json::json;
use sublime_fuzzy::best_match;
use wasm_bindgen::JsCast;
use yew;
use yewdux::mrc::Mrc;
// use router::{self, Route};
use crate::store::{LoginAction, PopupStore};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::rc::Rc;

use browser_rpass::log;
use serde;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};
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
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
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
    let on_create_submit = Callback::from({
        let path = props.path.clone();
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
    html! {
        <>
                    if props.user_id.is_some(){
                        <p>{format!("logged in as {}", (props.user_id).as_ref().unwrap())}</p>
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
                    accounts={Rc::new(accounts)}
                    />
                    </tbody>
                </table>
                <div>
                    // <button onclick={on_logout_click}>{ "Logout" }</button>
                    <form  onsubmit={on_create_submit}>
                    <label for="new-username">{"new username:"}</label>
                    <input type="text" id="new-username" name="create-username" ref={username_input_ref}/><br/>
                    <label for="new-password">{"new password:"}</label>
                    <input type="password" id="new-password" name="create-password" ref={password_input_ref}/><br/>
                    <button>{ "Create" }</button>
                    </form>
                </div>
    </>
    }
}
