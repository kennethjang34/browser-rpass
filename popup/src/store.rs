use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::types::{Account, User};
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Store)]
pub struct Store {
    pub user: Option<User>,
    pub page_loading: bool,
}
pub fn set_user(user: Option<User>, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.user = user;
    })
}
