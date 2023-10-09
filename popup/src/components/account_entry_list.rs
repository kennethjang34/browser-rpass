use std::rc::Rc;

use crate::Account;
use crate::Resource;
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
    let account_list_component = props
        .accounts
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, account)| {
            let delete_account2 = delete_account2.clone();
            let id = account.id.clone();
            html! {
                <tr key={id.clone()}>
                    <AccountEntry id={i} account={account.clone()}></AccountEntry>
                    <button onclick={move |e:MouseEvent|{delete_account2.emit(
                                (
                                e,account.clone())
                                )}}>{"delete"}</button>
                </tr>
            }
        })
        .collect::<Html>();
    account_list_component
}
