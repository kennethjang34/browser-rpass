use crate::{
    api::extension_api::logout,
    pages::*,
    store::{DataAction, LoginStatus, PopupStore},
    store::{PopupAction, StoreDataStatus},
    BoolState, BoolStateAction,
};
use browser_rpass::js_binding::extension_api::chrome;
use gloo_utils::{format::JsValueSerdeExt, window};
use log::*;
use wasm_bindgen::JsValue;

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
    let activated = use_selector(|state: &PopupStore| state.persistent_data.store_activated);
    let login_status = use_selector(|state: &PopupStore| match state.login_status {
        LoginStatus::LoggedIn | LoginStatus::LoginSuccess => true,
        _ => false,
    });

    let loading = use_selector(|state: &PopupStore| state.page_loading.clone());
    let path = use_selector(|state: &PopupStore| state.path.clone());
    let store_id = use_selector(|state: &PopupStore| state.persistent_data.store_id.clone());
    let store_ids = use_selector(|state: &PopupStore| state.store_ids.clone());
    let on_logout_click = Callback::from({
        move |event: MouseEvent| {
            event.prevent_default();
            // TODO logout for specific store only (currently logout is done for all stores)
            // logout((*store_id).clone())
            logout(None)
        }
    });
    let on_close = Callback::from(move |event: MouseEvent| {
        event.prevent_default();
        window().close().unwrap();
    });
    let dark_mode = use_selector(|state: &PopupStore| state.persistent_data.dark_mode);
    let set_darkmode = Callback::from(move |_| {
        Dispatch::<PopupStore>::new().apply(PopupAction::DarkModeToggle);
    });
    let show_create_store_popup = use_reducer(|| BoolState::new(false));
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let close_create_store_popup = {
        let show_create_store_popup = show_create_store_popup.clone();
        Callback::from({
            move |_: MouseEvent| {
                show_create_store_popup.dispatch(BoolStateAction::SetAction(false));
            }
        })
    };
    let close_toast = {
        let on_close = on_close.clone();
        Callback::from(move |_| {
            on_close.clone().emit(MouseEvent::new("click").unwrap());
        })
    };
    let create_store_clicked = {
        let show_create_store_popup = show_create_store_popup.clone();
        Callback::from(move |_: MouseEvent| {
            show_create_store_popup.dispatch(BoolStateAction::SetAction(true));
        })
    };

    html! {
                <div tabindex="-1" aria-hidden="true" style="width: 600px; height: 600px" class="top-0 left-0 overflow-hidden md:inset-0">
                   <div class="w-full h-full">
                      <div class="relative w-full h-full max-w-full max-h-full">
                         <div class="relative bg-white shadow dark:bg-gray-700 w-full h-full overflow-hidden">
                         <CloseButton onclick={on_close} class="absolute" style="z-index: 10; margin-top:0.5rem; margin-right:0.5rem;"/>
            if *store_status == StoreDataStatus::NativeAppConnectionError {

                <div>{format!("Native app connection error!!! maybe manifest file doesn't have the right extension id yet? By the way, your extension id is : {}",
                              js_sys::Reflect::get(&chrome.runtime(),&<JsValue as JsValueSerdeExt>::from_serde("id").unwrap())
                              .map(|v|v.as_string().unwrap_or("?".to_string())).map_err(|e|format!("(sorry could not find it myself.. last err returned: {})",e.as_string().unwrap())).unwrap())}</div>
            }else{                            <div class="px-3 py-1.5 lg:px-8 w-full h-full overflow-hidden">
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
                                  class="fixed top-5 my-6 right-0 mr-5 z-10 h-12 w-12 inline-flex cursor-pointer justify-center items-center rounded-lg p-2 hover:bg-gray-100 dark:hover:bg-gray-600">
                                  if *dark_mode {
                                      <SunIcon/>
                                  }
                                  else{
                                      <MoonIcon/>
                                  }

                              </button>
                                  if *login_status{
                                    <button  class="fixed my-3 bottom-0 right-0 mr-3 warning-btn" onclick={on_logout_click}>{"logout"}</button>
                                  }
                                    if store_id.is_some() && *activated{
                                        <AccountPage store_id={(*store_id.clone()).clone().unwrap()} path={(*path).clone()}/>
                                    }
                                else{
                                    <LoginPage/>
                                    if store_ids.len() == 0 && !show_create_store_popup.value{
                                    {
                                        html!{
                                            <div class="ms-3 text-sm font-normal absolute" style="
                                            display: flex;
                                            width: 100%;
                                            left: 0px;
                                            transform: translate(0%,0%);
                                            top: 0;
                                            height: 100%;
                                            background-color: white;
                                            align-items: center;
                                            z-index: 10000;
                                            align-content: center;
                                            justify-content: center;
                                            flex-wrap: wrap;
                                            flex-direction: column;
                                            ">
                <div class="mb-2 text-sm font-normal">{"No store exists. want to create one?"}</div>
                <div class="grid grid-cols-2 gap-2">
                    <div >
                        <button onclick={create_store_clicked.clone()} class="inline-flex justify-center w-full px-2 py-1.5 text-xs font-medium text-center text-white bg-blue-600 rounded-lg hover:bg-blue-700 focus:ring-4 focus:outline-none focus:ring-blue-300 dark:bg-blue-500 dark:hover:bg-blue-600 dark:focus:ring-blue-800">{"Create"}</button>
                    </div>
                    <div>
                        <button onclick={close_toast.clone()} class="inline-flex justify-center w-full px-2 py-1.5 text-xs font-medium text-center text-gray-900 bg-white border border-gray-300 rounded-lg hover:bg-gray-100 focus:ring-4 focus:outline-none focus:ring-gray-200 dark:bg-gray-600 dark:text-white dark:border-gray-600 dark:hover:bg-gray-700 dark:hover:border-gray-700 dark:focus:ring-gray-700">{"Not now"}</button>
                    </div>
                </div>
            </div>
                                        }
                                    }
                                }
                                }
                              if (*show_create_store_popup).into() {
                                    <CreateStorePopup handle_close={close_create_store_popup} style="height: 100%; width: 100%; z-index: 100000; background: white;"/>
                                }
                            </div>
            </div>

            }
            </div>
            </div>
            </div>
            </div>

    }
}
