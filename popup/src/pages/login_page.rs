use crate::api::extension_api::login;
use crate::components::FormInput;
use crate::store::{LoginAction, LoginStatus, PopupStore};
#[allow(unused_imports)]
use log::*;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;
use yew;

use serde;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
struct LoginSchema {
    // #[validate(
    //     length(min = 1, message = "Email is required"),
    //     email(message = "Email is invalid")
    // )]
    store_id: String,
}
#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(LoginPage)]
pub fn login_page(_props: &Props) -> Html {
    let (_popup_store, popup_store_dispatch) = use_store::<PopupStore>();
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));
    let store_id = use_state(|| popup_store_dispatch.get().persistent_data.store_id.clone());
    let handle_store_id_input = Callback::from({
        let store_id = store_id.clone();
        move |value: String| {
            store_id.set(Some(value));
        }
    });

    let remember_me = use_selector(|state: &PopupStore| state.persistent_data.remember_me);

    let handle_rememeber_me_input = Callback::from({
        let dispatch = popup_store_dispatch.clone();
        move |event: Event| {
            dispatch.apply(LoginAction::RememberMe(
                event.target_unchecked_into::<HtmlInputElement>().checked(),
            ));
        }
    });
    let _is_loading = use_selector(|state: &PopupStore| state.page_loading);
    let login_status = use_selector(|state: &PopupStore| state.login_status.clone());
    let on_submit = {
        let cloned_validation_errors = validation_errors.clone();
        let store_id = store_id.clone();
        let popup_store_dispatch = popup_store_dispatch.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let form = LoginSchema {
                store_id: (*store_id).clone().unwrap_or_default(),
            };
            let validation_errors = cloned_validation_errors.clone();
            let store_id = store_id.clone();

            match form.validate() {
                Ok(_) => {
                    let _form_data = form.clone();
                    popup_store_dispatch
                        .apply(LoginAction::LoginStarted(form.store_id.clone(), json!({})));
                    let store_id = (*store_id).clone().unwrap_or_default();
                    login(store_id);
                }
                Err(e) => {
                    validation_errors.set(Rc::new(RefCell::new(e)));
                }
            };
        })
    };
    let close_login_error = {
        let dispatch = popup_store_dispatch.clone();
        Callback::from(move |_| {
            dispatch.apply(LoginAction::LoginIdle);
        })
    };

    html! {
        <>

            <div class="mt-4 flex-col">
                <h3 class="text-xl font-medium text-gray-900 dark:text-white" style="text-align: center">{ "Login" }</h3>
                if *login_status == LoginStatus::LoginFailed || *login_status == LoginStatus::LoginError {
                    <div id="toast-danger" class="flex absolute right-0 items-center w-full max-w-xs p-4 mr-5 my-4 text-gray-500 bg-white rounded-lg shadow dark:text-gray-400 dark:bg-gray-800" role="alert">
                        <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 text-red-500 bg-red-100 rounded-lg dark:bg-red-800 dark:text-red-200">
                        <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/>
                        </svg>
                        <span class="sr-only">{"Error icon"}</span>
                        </div>
                        <div class="ms-3 text-sm font-normal">{"Login failed"}</div>
                        <button type="button" onclick={
                            close_login_error
                        } class="ms-auto -mx-1.5 -my-1.5 bg-white text-gray-400 hover:text-gray-900 rounded-lg focus:ring-2 focus:ring-gray-300 p-1.5 hover:bg-gray-100 inline-flex items-center justify-center h-8 w-8 dark:text-gray-500 dark:hover:text-white dark:bg-gray-800 dark:hover:bg-gray-700" data-dismiss-target="#toast-danger" aria-label="Close">
                    <span class="sr-only">{"Close"}</span>
                    <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                    <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                    </svg>
                    </button>
                    </div>
                }
                           <form
                              onsubmit={on_submit}
                                        class="space-y-6 m-2.5 relative" style="top:72px;" action="#"

                              >
                              <FormInput label="Email"  name="email" input_type="email"  handle_onchange={handle_store_id_input} errors={&*validation_errors} /* handle_on_input_blur={validate_input_on_blur.clone()} */
                              label_class={
    "block mb-auto text-sm font-medium text-gray-900 dark:text-white"
                              }
        input_class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white" placeholder={"name@company.com"}
                              value={(*store_id).clone()}
                              />
                                  <div class="flex justify-between">
                        <div class="flex items-start">
                            <div class="flex items-center h-5">
                                <input id="remember" type="checkbox" checked={*remember_me} class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800" required={false} onchange={handle_rememeber_me_input}/>
                            </div>
                            <label for="remember" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{"Remeber me"}</label>
                        </div>
                        <a href="#" class="text-sm text-blue-700 hover:underline dark:text-blue-500">{"Forgot password?"}</a>
                    </div>
                                                      <button type="submit" class="w-full text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">{"Login to your account"}</button>
                                                      <div class="text-sm font-medium text-gray-500 dark:text-gray-300">
                                                      {"Not registered? "} <a href="#" class="text-blue-700 hover:underline dark:text-blue-500">{"Create new account"}</a>
                    </div>

                           </form>
            </div>
    </>
    }
}
