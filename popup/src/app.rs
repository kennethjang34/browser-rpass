use crate::{
    api::types::Account,
    components::{account_entry_list::AccountEntryList, *},
};
use account_entry::AccountEntry;
use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    let mock_accounts = vec![
        Account {
            id: 1,
            username: Some("mu1".to_owned()),
            email: "mu1@gmail.com".to_owned(),
            password: Some("abc".to_owned()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            organization: Some("apple company".to_owned()),
        },
        Account {
            id: 2,
            username: None,
            email: "mu2@gmail.com".to_owned(),
            password: Some("def".to_owned()),
            created_at: Some(chrono::Utc::now()),
            updated_at: Some(chrono::Utc::now()),
            organization: Some("banana company".to_owned()),
        },
    ];

    html! {
        <div>
                <label for="account-search">{"Search for account:"}</label><br/>
                <input type="search" id="account-search" name="account-search"/>
            <button>{ "Search" }</button>
            <table class="table table-bordered">
                <thead>
                  <tr>
                    <th>{ "ID" }</th>
                    <th>{ "Password" }</th>
                  </tr>
                </thead>
                <tbody>
                <AccountEntryList accounts={mock_accounts.clone()}/>
                </tbody>
            </table>
        </div>
    }
}
