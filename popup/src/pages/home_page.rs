use crate::{
    api::extension_api::{fetch_accounts, logout},
    pages::account_page::AccountPage,
    pages::login_page::LoginPage,
    store::PopupStore,
    store::StoreDataStatus,
};
use gloo_utils::window;
use log::*;
use std::rc::Rc;

use crate::components::CloseButton;
use crate::components::LoadingIndicator;
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
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    html! {
            <div tabindex="-1" aria-hidden="true" style="width: 600px; height: 600px" class="top-0 left-0 right-0 overflow-hidden md:inset-0">
               <div class="w-full h-full">
                  <div class="relative w-full h-full max-w-full max-h-full">
                     <div class="relative bg-white shadow dark:bg-gray-700 w-full h-full overflow-hidden">
                     <CloseButton onclick={on_close} class="absolute" style="z-index: 10;"/>
                        <div class="px-3 py-1.5 lg:px-8 w-full h-full overflow-hidden">
                        if *loading {
                            <div class="absolute flex items-center justify-center rounded-lg  overflow-hidden" style="left: 50%; top:50%;transform: translate(-50%,-50%);">
                                <LoadingIndicator class={"mr-2 "}/>
                            </div>
                        }
                        if *store_status == StoreDataStatus::FetchFailed{
                            <div class="absolute flex items-center justify-center rounded-lg  overflow-hidden p-4 mb-4 text-sm text-red-800 rounded-lg bg-red-50 dark:bg-gray-800 dark:text-red-400 z-50"
                                style="left: 50%; top:50%;transform: translate(-50%,-50%);" role="alert">
                                <svg class="flex-shrink-0 inline w-4 h-4 me-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                                    <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5ZM9.5 4a1.5 1.5 0 1 1 0 3 1.5 1.5 0 0 1 0-3ZM12 15H8a1 1 0 0 1 0-2h1v-3H8a1 1 0 0 1 0-2h2a1 1 0 0 1 1 1v4h1a1 1 0 0 1 0 2Z"/>
                                </svg>
                                <span class="sr-only">{"error"}</span>
                                <div>
                                    <span class="font-medium">{"Error!"}</span> {"Data Loading Failed"}
                                </div>
                            </div>
                        }
                            <div style={
                                "height:90%;".to_owned()+
                                {
                                    if *loading || *store_status == StoreDataStatus::FetchFailed{
                                        "opacity: 0.5; pointer-events: none"
                                    }
                                    else{
                                        "opacity: 1;"
                                    }
                                }
                            }>
                                if *verified{
                                    <AccountPage user_id={(*user_id).clone()} path={(*path).clone()}/>
                                    <button type="button" class="fixed my-3 bottom-0 right-0 mr-3 warning-btn" onclick={on_logout_click}>{"logout"}</button>
                                }
                                else{
                                    <LoginPage />
                                }
                            </div>
        </div>
        </div>
        </div>
        </div>
        </div>
    }
}
