use wasm_bindgen::prelude::*;
#[macro_use]
mod util;

cfg_if! {
    if #[cfg(feature = "console_log")] {
        fn init_log() {
            let _=browser_rpass::setup_logger();
        }
    } else {
        fn init_log() {}
    }
}

#[wasm_bindgen(start)]
pub async fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    init_log();
}
