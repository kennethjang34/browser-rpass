use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;

use crate::api::types::{Account, User};
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Store)]
pub struct Store {
    pub user: Option<User>,
    pub page_loading: bool,
    pub alert_input: AlertInput,
}
pub fn set_user(user: Option<User>, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.user = user;
    })
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AlertInput {
    pub show_alert: bool,
    pub alert_message: String,
}

pub fn set_page_loading(loading: bool, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.page_loading = loading;
    });
}
// pub fn set_auth_user(user: Option<User>, dispatch: Dispatch<Store>) {
//     dispatch.reduce_mut(move |store| {
//         store.auth_user = user;
//     })
// }

pub fn set_show_alert(message: String, dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.alert_input = AlertInput {
            alert_message: message,
            show_alert: true,
        };
    })
}

pub fn set_hide_alert(dispatch: Dispatch<Store>) {
    dispatch.reduce_mut(move |store| {
        store.alert_input.show_alert = false;
    })
}

use browser_rpass::util::{chrome, Port};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EXTENSION_PORT: Port = chrome.runtime().connect();
}
