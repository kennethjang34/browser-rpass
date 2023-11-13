use crate::components::edit_account_popup::EditAccountPopup;
use crate::Account;
use crate::Resource;
use log::*;
use std::rc::Rc;
use yew;
use yew::prelude::*;

use crate::api::extension_api::delete_resource;

use super::account_entry::AccountEntry;

#[derive(Debug, PartialEq, Properties, Clone)]
pub struct AccountEntryListProps {
    pub accounts: Rc<Vec<Rc<Account>>>,
}

#[function_component(AccountEntryList)]
pub fn account_entry_list_component(props: &AccountEntryListProps) -> Html {
    let delete_account = {
        Callback::<(MouseEvent, Rc<Account>)>::from({
            move |(e, account): (MouseEvent, Rc<Account>)| {
                e.prevent_default();
                let id = account.id.clone();
                delete_resource(id.clone(), Resource::Account);
            }
        })
    };
    let delete_account2 = delete_account.clone();
    let show_edit_account = use_state(|| false);
    let on_edit_account = Callback::<MouseEvent>::from({
        let show_edit_account = show_edit_account.clone();
        move |e: MouseEvent| {
            e.prevent_default();
            show_edit_account.set(true);
        }
    });
    let close_edit_account_popup = {
        let show_edit_account = show_edit_account.clone();
        Callback::from(move |_: MouseEvent| {
            show_edit_account.set(false);
        })
    };
    let account_list_component = props
        .accounts
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, account)| {
            let delete_account2 = delete_account2.clone();
            let id = account.id.clone();
            let account2 = account.clone();
            html! {
            <>
                <tr key={id.clone()} class="bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600">
                    <AccountEntry id={i} account={account.clone()}></AccountEntry>
                    <td class="px-1 py-0.5">
                    <a href="#" 
                    onclick={
                        on_edit_account.clone()
                    }
                    class="font-medium text-blue-600 dark:text-blue-500 hover:underline">{ "Edit" }</a>
                    </td>
                    <td class="px-1 py-0.5">
                    <a href="#" 
                    onclick={move |e:MouseEvent|{delete_account2.emit(
                                (
                                e,account2.clone())
                    )}}
                    class="font-medium text-blue-600 dark:text-blue-500 hover:underline">{ "Delete" }</a>
                    </td>
                </tr>
                    if *show_edit_account{
                        <EditAccountPopup account={account.clone()} handle_close={close_edit_account_popup.clone()}/>
                    }
            </>
            }
        })
        .collect::<Html>();
    account_list_component
}
