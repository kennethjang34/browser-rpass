use std::{collections::HashMap, sync::Mutex};

use browser_rpass::util::{chrome, Port};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref NATIVE_PORT: Port = chrome.runtime().connect_native("com.rpass");
}
