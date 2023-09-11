use std::{any::type_name, collections::HashMap, future::Future, pin::Pin, rc::Rc, sync::Mutex};

use crate::{
    request::RequestEnum,
    response::ResponseEnum,
    util::{chrome, Port},
};
use gloo::{console::log, storage::errors::StorageError};
use lazy_static::lazy_static;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use yewdux::prelude::init_listener;
use yewdux::{
    prelude::{Dispatch, Listener},
    store::Store,
};
pub type AsyncCallback =
    Box<dyn Send + FnOnce(ResponseEnum, Port) -> Pin<Box<dyn Future<Output = ()>>>>;

lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_NATIVE: Mutex<HashMap<String, Box<dyn FnOnce(&[u8], Port) -> Result<(), String> + Send>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_POP_UP: Mutex<HashMap<String,AsyncCallback>>
    // Box<dyn Send+FnOnce<Fut>(ResponseEnum, Port)->Fut where Fut:Future<Output=()>>>>
        =
        // impl Future<Output=()> + Send>>> =
        // Result<(), String> + Send>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref DATA_STORAGE: Mutex<HashMap<String, String>> = {
        let mut map = HashMap::new();
        Mutex::new(map)
    };
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionAction {
    Login(bool),
    LoginError,
    Logout,
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionEvent {
    LoginSucceeded,
    LoginFailed,
    LoginError,
    LogoutSucceeded,
    LogoutFailed,
}
