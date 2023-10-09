use crate::{components::header::Header, store::PopupStore};
use wasm_bindgen_futures;
use yew;
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component(ProfilePage)]
pub fn profile_page() -> Html {
    let (_store, dispatch) = use_store::<PopupStore>();

    use_effect_with_deps(
        move |_| {
            let _dispatch = dispatch.clone();
            wasm_bindgen_futures::spawn_local(async move {});
        },
        (),
    );

    html! {
    <>
      <Header />
      <section class="bg-ct-blue-600 min-h-screen pt-20">
        <div class="max-w-4xl mx-auto bg-ct-dark-100 rounded-md h-[20rem] flex justify-center items-center">
          <div>
            <p class="text-5xl font-semibold">{"Profile Page"}</p>
          </div>
        </div>
      </section>
    </>
    }
}
