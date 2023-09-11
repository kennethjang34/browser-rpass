use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::{api::types::Account, store::PopupStore};

use super::account_entry::AccountEntry;

#[derive(Debug, PartialEq, Properties)]
pub struct AccountEntryListProps {
    pub accounts: Vec<Account>,
}

#[function_component(AccountEntryList)]
pub fn account_entry_list_component(props: &AccountEntryListProps) -> Html {
    let (store, dispatch) = use_store::<PopupStore>();
    let accounts = &props.accounts;
    accounts
        .iter()
        .map(|account| {
            html! {
                <tr>
                    <AccountEntry id={account.id.clone()} account={account.clone()}></AccountEntry>
                </tr>
            }
        })
        .collect::<Html>()
}
