#[allow(warnings)]
pub mod request;
pub mod response;
pub mod store;
pub mod util;
pub enum StringOrCallback {
    String(String),
    Callback(Box<dyn FnOnce() -> ()>),
}
pub use yewdux::prelude::use_store;
