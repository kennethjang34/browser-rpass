pub mod request;
pub mod response;
pub mod store;
pub mod util;
pub enum StringOrCallback {
    String(String),
    Callback(Box<dyn FnOnce() -> ()>),
}
