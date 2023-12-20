use crate::{
    api::extension_api::{fetch_accounts, logout},
    pages::account_page::AccountPage,
    pages::login_page::LoginPage,
    store::PopupStore,
    store::{PopupAction, StoreDataStatus},
};
use gloo_utils::window;
use log::*;
use std::rc::Rc;

use crate::components::*;
use yew;
use yew::prelude::*;
use yewdux::{self};
use yewdux::{dispatch::Dispatch, prelude::use_selector};

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(HomePage)]
pub fn home_page(_props: &Props) -> Html {
    trace!("render home page");
    let verified = use_selector(|state: &PopupStore| state.verified);
    let loading = use_selector(|state: &PopupStore| state.page_loading.clone());
    let path = use_selector(|state: &PopupStore| state.path.clone());
    let store_id = use_selector(|state: &PopupStore| state.persistent_data.store_id.clone());
    let on_logout_click = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        logout();
    });
    let on_close = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        window().close().unwrap();
    });
    use_effect_with(verified.clone(), {
        let _path = path.clone();
        move |verified: &Rc<bool>| {
            if **verified {
                fetch_accounts(None);
            }
        }
    });
    let dark_mode = use_selector(|state: &PopupStore| state.persistent_data.dark_mode);
    let set_darkmode = Callback::from(move |_| {
        Dispatch::<PopupStore>::new().apply(PopupAction::DarkModeToggle);
    });
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    html! {
            <div tabindex="-1" aria-hidden="true" style="width: 600px; height: 600px" class="top-0 left-0 overflow-hidden md:inset-0">
               <div class="w-full h-full">
                  <div class="relative w-full h-full max-w-full max-h-full">
                     <div class="relative bg-white shadow dark:bg-gray-700 w-full h-full overflow-hidden">
                     <CloseButton onclick={on_close} class="absolute" style="z-index: 10;"/>
                        <div class="px-3 py-1.5 lg:px-8 w-full h-full overflow-hidden">
                        if *loading {
                            <div class="absolute flex items-center justify-center rounded-lg  overflow-hidden center-position" >
                                <LoadingIndicator class={"mr-2 "}/>
                            </div>
                        }
                        if *store_status == StoreDataStatus::FetchFailed {
                            <div class="absolute mb-4 z-50 critical-error center-position"
                                role="alert">
                                <ErrorIcon/>
                                <span class="sr-only">{"error"}</span>
                                <div>
                                    <span class="font-medium">{"Error!"}</span>{" Data Loading Failed"}
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
                          <button onclick={set_darkmode.clone()}
                              class="fixed my-6 right-0 mr-5 z-10 h-12 w-12 inline-flex cursor-pointer justify-center items-center rounded-lg p-2 hover:bg-gray-100 dark:hover:bg-gray-600">
                              if *dark_mode {
                                  <SunIcon/>
                              }
                              else{
                                  <MoonIcon/>
                              }
                          </button>
                            if *verified{
                                <AccountPage store_id={(*store_id).clone()} path={(*path).clone()}/>
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
