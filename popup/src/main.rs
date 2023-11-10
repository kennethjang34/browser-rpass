use popup::run_app;

fn main() {
    wasm_bindgen_futures::spawn_local(async {
        let _ = run_app().await;
    });
}
