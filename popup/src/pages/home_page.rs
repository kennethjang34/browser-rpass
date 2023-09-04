use crate::pages::login_page::LoginPage;
use std::ops::Deref;

use crate::{
    api::types::Account,
    components::{account_entry_list::AccountEntryList, header::Header},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    #[prop_or_default]
    pub passphrase: Option<String>,
}
#[function_component(HomePage)]
pub fn home_page(props: &Props) -> Html {
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
    // let passphrase_state = use_state(|| None);
    // let passphrase_state_cloned = passphrase_state.clone();
    html! {
        <div>
            // if let Some(passphrase) = passphrase_state.deref().clone(){
            if let Some(ref passphrase) = props.passphrase{
                <p class="mb-4">{format!("Passphrase: {}", passphrase)}</p>
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
            }else{
                <p>{"need to login!"}</p>
                    <LoginPage />
            }
        </div>
    }
}
