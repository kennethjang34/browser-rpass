pub mod request;
pub mod response;
pub mod store;
pub mod types;
pub mod util;
pub enum StringOrCallback {
    String(String),
    Callback(Box<dyn FnOnce() -> ()>),
}
pub use console_error_panic_hook;
pub use getrandom;
pub use rand;
pub use typetag;
pub use url;
pub use util::*;
#[macro_export]
macro_rules! dbg {
    () => {
        log::debug!("[{}:{}]", file!(), line!());
    };
    ($val:expr) => {
        match $val {
            tmp => {
                log::debug!("{} = {:#?}",
                    stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($val:expr,) => { dbg!($val) };
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}
