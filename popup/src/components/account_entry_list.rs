use std::rc::Rc;

use crate::Account;
use crate::Resource;
use log::*;
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
    let edit_account = {
        Callback::<(MouseEvent, Rc<Account>)>::from({
            move |(e, account): (MouseEvent, Rc<Account>)| {
                e.prevent_default();
                let id = account.id.clone();
                debug!("edit account: {:?}", account);
                // delete_resource(id.clone(), Resource::Account);
            }
        })
    };
    let account_list_component = props
        .accounts
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, account)| {
            let delete_account2 = delete_account2.clone();
            let edit_account = edit_account.clone();
            let id = account.id.clone();
            let account2 = account.clone();
            html! {
                <tr key={id.clone()} class="bg-white border-b dark:bg-gray-800 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-600">
                    <AccountEntry id={i} account={account.clone()}></AccountEntry>
                    <td class="px-1 py-0.5">
                    <a href="#" 
                    onclick={move |e:MouseEvent|{
                        edit_account.emit(
                                (
                        e,account.clone())
                    )}}
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
            }
        })
        .collect::<Html>();
    account_list_component
}
