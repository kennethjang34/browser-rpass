use crate::components::EditAccountPopup;
use crate::store::DataAction;
use crate::store::PopupStore;
use crate::Account;
use crate::Resource;
#[allow(unused_imports)]
use log::*;
use std::rc::Rc;
use yew;
use yew::prelude::*;
use yewdux::dispatch::Dispatch;

use crate::api::extension_api::delete_resource;

use super::account_entry::AccountEntry;

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct AccountEntryListProps {
    pub accounts: Rc<Vec<Rc<Account>>>,
    pub store_id: String,
}

#[function_component(AccountEntryList)]
pub fn account_entry_list_component(props: &AccountEntryListProps) -> Html {
    let delete_account = {
        let store_id = props.store_id.clone();
        Callback::<(MouseEvent, Rc<Account>)>::from({
            move |(e, account): (MouseEvent, Rc<Account>)| {
                e.prevent_default();
                let id = account.id.clone();
                delete_resource(id.clone(), Resource::Account, Some(store_id.clone()));
            }
        })
    };
    let show_edit_account = use_state(|| None);
    let on_edit_account = Callback::<(MouseEvent, Rc<Account>)>::from({
        let show_edit_account = show_edit_account.clone();
        move |(e, account): (MouseEvent, Rc<Account>)| {
            e.prevent_default();
            show_edit_account.set(Some(account.clone()));
        }
    });
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    let close_edit_account_popup = {
        let show_edit_account = show_edit_account.clone();
        Callback::from(move |_: MouseEvent| {
            show_edit_account.set(None);
            popup_store_dispatch.apply(DataAction::Idle);
        })
    };
    let account_list_component = props
        .accounts
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, account)| {
            let delete_account = delete_account.clone();
            let id = account.id.clone();
            let account = account.clone();
            let on_edit_account = on_edit_account.clone();
            html! {
                <tr key={id.clone()} class="table-row">
                    <AccountEntry id={i} account={account.clone()}></AccountEntry>
                    <td class="px-1 py-0.5 text-center">
                        <a href="#"
                            onclick={
                                let account=account.clone();
                                move |e:MouseEvent|{
                                    on_edit_account.emit((e,account.clone()))
                                }
                            }
                            class="font-medium text-blue-600 dark:text-blue-500 hover:underline">
                            { "Edit" }
                        </a>
                    </td>
                    <td class="px-1 py-0.5 text-center">
                        <a href="#"
                            onclick={
                                move |e:MouseEvent|{
                                    delete_account.emit((e,account.clone()))
                                }
                            }
                            class="font-medium text-red-600 dark:text-red-500 hover:underline">
                            { "Delete" }
                        </a>
                    </td>
                </tr>
            }
        })
        .collect::<Html>();
    html! {
        <>
            {account_list_component}
            if let Some(account) = (*show_edit_account).clone(){
                <div class="fullscreen-container">
                    <EditAccountPopup account={account.clone()} handle_close={close_edit_account_popup.clone()} store_id={props.store_id.clone()}/>
                </div>
            }
        </>
    }
}
