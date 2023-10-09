use wasm_bindgen_futures::spawn_local;
use yew;
use yew::prelude::*;
// use yew_router::prelude::*;
use yewdux::prelude::*;

use crate::store::PopupStore;

#[function_component(Header)]
pub fn header_component() -> Html {
    let (_store, dispatch) = use_store::<PopupStore>();
    // let user = store.auth_user.clone();

    let _handle_logout = {
        let store_dispatch = dispatch.clone();

        Callback::from(move |_: MouseEvent| {
            let _dispatch = store_dispatch.clone();
            spawn_local(async move {
                // set_page_loading(true, dispatch.clone());
                // let res = api_logout_user().await;
                // match res {
                //     Ok(_) => {
                //         set_page_loading(false, dispatch.clone());
                //         set_auth_user(None, dispatch.clone());
                //         set_show_alert("Logged out successfully".to_string(), dispatch);
                //     }
                //     Err(e) => {
                //         set_show_alert(e.to_string(), dispatch.clone());
                //         set_page_loading(false, dispatch);
                //     }
                // };
            });
        })
    };

    html! {
        <header class="bg-white h-20">
        <nav class="h-full flex justify-between container items-center">
          <div>
            // <Link<Route> to={Route::HomePage} classes="text-ct-dark-600">{"Home"}</Link<Route>>
          </div>
          <ul class="flex items-center gap-4">
            <li>
              // <Link<Route> to={Route::HomePage} classes="text-ct-dark-600">{"Home"}</Link<Route>>
            </li>
            // if user.is_some() {
            //    <>
            //     <li>
            //       <Link<Route> to={Route::ProfilePage} classes="text-ct-dark-600">{"Profile"}</Link<Route>>
            //     </li>
            //     <li
            //       class="cursor-pointer"
            //     >
            //       {"Create Post"}
            //     </li>
            //     <li class="cursor-pointer" onclick={handle_logout}>
            //       {"Logout"}
            //     </li>
            //   </>
            //
            // } else {
              <>
                // <li>
                  // <Link<Route> to={Route::RegisterPage} classes="text-ct-dark-600">{"SignUp"}</Link<Route>>
                // </li>
                <li>
                  // <Link<Route> to={Route::LoginPage} classes="text-ct-dark-600">{"Login"}</Link<Route>>
                </li>
              </>
            // }
          </ul>
        </nav>
      </header>
    }
}
