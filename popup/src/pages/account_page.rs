use crate::store::{DataAction, LoginAction, LoginStatus, PopupStore, StoreDataStatus};
use crate::{components::*, BoolState, BoolStateAction};
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
    pub store_id: String,
    pub path: Option<String>,
}

#[function_component(AccountPage)]
pub fn account_page(props: &Props) -> Html {
    trace!("account page");
    let search_string = use_state(|| String::new());
    let path = props.path.clone();
    let account_selector = use_selector_with_deps(
        |state: &PopupStore, (search_string, path)| {
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
            move |_: MouseEvent| {
                show_create_account_popup.set(false);
            }
        })
    };
    let store_switcher_visible = use_reducer(BoolState::default);
    let login_status = use_selector(|state: &PopupStore| state.login_status.clone());
    let close_store_switcher = {
        let store_switcher_visible = store_switcher_visible.clone();
        Callback::from({
            move |_: MouseEvent| {
                store_switcher_visible.dispatch(BoolStateAction::SetAction(false));
            }
        })
    };
    let close_toast = {
        let dispatch = store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };
    let table_header_element = move |text: &str| -> Html {
        html! {
            <th scope="col">
                {
                    text
                }
            </th>
        }
    };

    let current_store_id =
        use_selector(|state: &PopupStore| state.persistent_data.store_id.clone());
    let on_switch_stores = {
        let store_switcher_visible = store_switcher_visible.clone();
        Callback::from({
            move |_| {
                store_switcher_visible.dispatch(BoolStateAction::ToggleAction);
            }
        })
    };
    let table_headers = ["username", "password", "note", "", ""];
    if *login_status == LoginStatus::LoginSuccess {
        store_dispatch.apply(LoginAction::LoggedIn);
        store_switcher_visible.dispatch(BoolStateAction::SetAction(false));
    }
    let show_create_store_popup = use_reducer(|| BoolState::new(false));
    let show_delete_store_popup = use_reducer(|| BoolState::new(false));
    let on_create_store = Callback::from({
        let show_create_store_popup = show_create_store_popup.clone();
        move |event: MouseEvent| {
            event.prevent_default();
            show_create_store_popup.dispatch(BoolStateAction::ToggleAction);
        }
    });
    let close_create_store_popup = {
        let show_create_store_popup = show_create_store_popup.clone();
        Callback::from({
            move |_: MouseEvent| {
                show_create_store_popup.dispatch(BoolStateAction::SetAction(false));
            }
        })
    };
    let on_delete_store = Callback::from({
        let show_delete_store_popup = show_delete_store_popup.clone();
        move |event: MouseEvent| {
            event.prevent_default();
            show_delete_store_popup.dispatch(BoolStateAction::ToggleAction);
        }
    });
    let close_delete_store_popup = {
        let show_delete_store_popup = show_delete_store_popup.clone();
        Callback::from({
            move |_: MouseEvent| {
                show_delete_store_popup.dispatch(BoolStateAction::SetAction(false));
            }
        })
    };
    html! {
        <>
            <div class="relative overflow-hidden shadow-md sm:rounded-lg w-full h-full">
            <div class="w-full top-2.5" style="border-bottom:outset; height: 80%;">
            <SearchInput onchange={on_search} value={(*search_string).clone()}/>
        <div class={classes!("flex")}>
            <button  class="primary-btn block  my-4 mx-2" type="button" onclick={&on_switch_stores}>
                {"Switch stores"}
            </button>
            <button type="button" class="my-4 mx-2 accent-btn" onclick={on_create_store}>{"create store"}</button>
            <button type="button" class="my-4 mx-2 warning-btn" onclick={on_delete_store}>{"delete store"}</button>
                </div>
                        if (*show_create_store_popup).into(){
                            <div class="fullscreen-container">
                                <CreateStorePopup handle_close={close_create_store_popup}/>
                            </div>
                        }
                        if (*show_delete_store_popup).into(){
                            <div class="fullscreen-container">
                                <DeleteStorePopup handle_close={close_delete_store_popup}/>
                            </div>
                        }
                        if show_create_store_popup.value == false && show_delete_store_popup.value==false {
                            if let StoreDataStatus::StoreCreationFailed(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Error} on_close_button_clicked={close_toast.clone()} text={format!("Failed to create store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            if let StoreDataStatus::StoreCreated(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Success} on_close_button_clicked={close_toast.clone()} text={format!("Successfully created store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            if let StoreDataStatus::StoreDeletionFailed(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Error} on_close_button_clicked={close_toast.clone()} text={format!("Failed to delete store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            if let StoreDataStatus::StoreDeleted(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Success} on_close_button_clicked={close_toast.clone()} text={format!("Successfully deleted store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                        }


                    if *store_status==StoreDataStatus::DeletionFailed{
                        <Toast toast_type={ToastType::Error} on_close_button_clicked={close_toast.clone()} text={"Deletion Failed"} class="absolute right-0 top-5 z-10"/>
                    }
                    if *store_status==StoreDataStatus::DeletionSuccess{
                        <Toast toast_type={ToastType::Success} on_close_button_clicked={close_toast.clone()} text={"Deletion Success"} class="absolute right-0 top-5 z-10"/>
                    }
            <div class={classes!("h-72", "overflow-y-auto")}>
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
                        <AccountEntryList accounts={account_selector} store_id={props.store_id.clone()}/>
                    </tbody>
                </table>
                </div>
                </div>
                <div style="display: flex; align-items: center; justify-content: space-between;">
                    <button  class="primary-btn block  my-4 mx-2" type="button" onclick={on_create_account}>
                    {"Create Account"}
                    </button>
                    if let Some(current_store_id)=(*current_store_id).clone(){
                        <span class="p-1.5 dark:text-white">{"currently in "}
                            <span class="text-blue-700 dark:text-blue-400">{current_store_id}</span>
                        </span>
                    }
                </div>
                if *show_create_account_popup{
                    <div class="fullscreen-container">
                        <CreateAccountPopup domain={props.path.clone()} handle_close={close_create_account_popup} store_id={props.store_id.clone()}/>
                    </div>
                }
            if (*store_switcher_visible).into() {
            <div class="fullscreen-container">
                <SimplePopup handle_close={&close_store_switcher}>
                    <StoreSwitcher/>
                </SimplePopup>
            </div>
            }
            </div>
        </>
    }
}
