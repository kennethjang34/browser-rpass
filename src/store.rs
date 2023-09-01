use std::{collections::HashMap, sync::Mutex};

use crate::{response::ResponseEnum, util::Port};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_NATIVE: Mutex<HashMap<String, Box<dyn FnOnce(&[u8], Port) -> Result<(), String> + Send>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref MESSAGE_ACKNOWLEDGEMENTS_POP_UP: Mutex<HashMap<String, Box<dyn FnOnce(ResponseEnum, Port) -> Result<(), String> + Send>>> =
        Mutex::new(HashMap::new());
}
lazy_static! {
    pub static ref DATA_STORAGE: Mutex<HashMap<String, String>> = {
        let mut map = HashMap::new();
        // map.insert("passphrase".to_owned(), "abcd".to_owned());
        Mutex::new(map)
    };
}
