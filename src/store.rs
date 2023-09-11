use std::{collections::HashMap, future::Future, pin::Pin, sync::Mutex};

use crate::{
    request::RequestEnum,
    response::ResponseEnum,
    util::{chrome, Port},
};
use lazy_static::lazy_static;
pub type AsyncCallback =
    Box<dyn Send + FnOnce(ResponseEnum, Port) -> Pin<Box<dyn Future<Output = ()>>>>;

lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_NATIVE: Mutex<HashMap<String, Box<dyn FnOnce(&[u8], Port) -> Result<(), String> + Send>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_POP_UP: Mutex<HashMap<String,AsyncCallback>>
        =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref DATA_STORAGE: Mutex<HashMap<String, String>> = {
        let mut map = HashMap::new();
        Mutex::new(map)
    };
}
