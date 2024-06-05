use std::sync::{Arc, Mutex};

pub use rpass::interface::*;
use rpass::pass::PasswordStore;

pub type PasswordStoreType = Arc<Mutex<Arc<Mutex<PasswordStore>>>>;
/// The list of stores that the user have.
pub type StoreListType = Arc<Mutex<Vec<Arc<Mutex<PasswordStore>>>>>;

pub mod request_handler;
pub mod store_api;
pub mod util;
