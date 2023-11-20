use crate::{
    api::extension_api::{create_account, fetch_accounts, logout},
    pages::account_page::AccountPage,
    pages::login_page::LoginPage,
    store::PopupStore,
    Account,
};
use gloo_utils::window;
use log::*;
use std::rc::Rc;

use yew;
use yew::prelude::*;
use yewdux::prelude::use_selector;
use yewdux::{self};

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(HomePage)]
pub fn home_page(_props: &Props) -> Html {
    trace!("render home page");
    let verified = use_selector(|state: &PopupStore| state.verified);
    let loading = use_selector(|state: &PopupStore| state.page_loading.clone());
    let path = use_selector(|state: &PopupStore| state.path.clone());
    let user_id = use_selector(|state: &PopupStore| state.persistent_data.user_id.clone());
    let on_logout_click = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        logout();
    });
    let on_close = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        window().close().unwrap();
    });
    use_effect_with_deps(
        {
            let _path = path.clone();
            move |verified: &Rc<bool>| {
                if **verified {
                    fetch_accounts(None);
                }
            }
        },
        verified.clone(),
    );
    html! {
                <div tabindex="-1" aria-hidden="true" style="width: 500px; height: 500px" class=" top-0 left-0 right-0 z-50 overflow-hidden md:inset-0"
                >
                   <div class="w-full h-full">
                      <div class="relative w-full h-full max-w-full max-h-full">
                         <div class="relative bg-white shadow dark:bg-gray-700 w-full h-full overflow-hidden">
                            <button type="button" class="absolute bg-transparent dark:hover:bg-gray-600 dark:hover:text-white h-6 hover:bg-gray-200 hover:text-gray-900 inline-flex items-center justify-center ml-auto right-2.5 rounded-lg text-gray-400 text-sm w-6" style="z-index: 1000;" onclick={on_close}>
                               <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                                  <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                               </svg>
                               <span class="sr-only">{"close modal"}</span>
                            </button>
                            <div class="px-3 py-1.5 lg:px-8 w-full h-full overflow-hidden">

            if *loading {
                    <div class="flex items-center justify-center w-full h-full border border-gray-200 rounded-lg bg-gray-50 dark:bg-gray-800 dark:border-gray-700 overflow-hidden">
        <div role="status">
            <svg aria-hidden="true" class="w-full h-full mr-2 text-gray-200 animate-spin dark:text-gray-600 fill-blue-600" viewBox="0 0 100 101" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z" fill="currentColor"/><path d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z" fill="currentFill"/></svg>
            <span class="sr-only">{"Loading..."}</span>
        </div>
    </div>
            }else{
                if *verified{
                    <AccountPage user_id={(*user_id).clone()} path={(*path).clone()}/>
                    <button type="button" class="fixed my-3 bottom-0 right-0 text-red-700 hover:text-white border border-red-700 hover:bg-red-800 focus:ring-4 focus:outline-none focus:ring-red-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center mr-3 dark:border-red-500 dark:text-red-500 dark:hover:text-white dark:hover:bg-red-600 dark:focus:ring-red-900" onclick={on_logout_click}>{"logout"}</button>
                }
                else{
                        <LoginPage />
                }
            }
            </div>
            </div>
            </div>
            </div>
            </div>
        }
}
