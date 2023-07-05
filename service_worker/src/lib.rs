use wasm_bindgen::prelude::*;
#[macro_use]
mod util;

#[wasm_bindgen(start)]
pub async fn main() {
    log!("inside main");
}
