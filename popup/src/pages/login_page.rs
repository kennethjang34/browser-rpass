use crate::api::extension_api::login;
// use crate::api::user_api::api_login_user;
use crate::components::{form_input::FormInput, loading_button::LoadingButton};
use log::*;
use serde_json::json;
use wasm_bindgen::JsCast;
use yew;
// use router::{self, Route};
use crate::store::{LoginAction, PopupStore};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use browser_rpass::log;
use serde;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
struct LoginUserSchema {
    // #[validate(
    //     length(min = 1, message = "Email is required"),
    //     email(message = "Email is invalid")
    // )]
    user_id: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        // length(min = 6, message = "Password must be at least 6 characters")
    )]
    passphrase: String,
}
#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(LoginPage)]
pub fn login_page(_props: &Props) -> Html {
    let (_popup_store, popup_store_dispatch) = use_store::<PopupStore>();
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));
    let user_id = use_state(|| popup_store_dispatch.get().persistent_data.user_id.clone());
    let passphrase = use_state(|| String::new());
    let handle_user_id_input = Callback::from({
        let user_id = user_id.clone();
        move |value: String| {
            user_id.set(Some(value));
        }
    });
    let handle_passphrase_input = {
        let passphrase = passphrase.clone();
        Callback::from(move |value| {
            passphrase.set(value);
        })
    };

    let validate_input_on_blur = {
        let cloned_validation_errors = validation_errors.clone();
        Callback::from({
            let user_id = user_id.clone();
            let passphrase = passphrase.clone();
            move |(name, _value): (String, String)| {
                let login_form = LoginUserSchema {
                    user_id: (*user_id).clone().unwrap_or_default(),
                    passphrase: (*passphrase).clone(),
                };

                match login_form.validate() {
                    Ok(_) => {
                        cloned_validation_errors
                            .borrow_mut()
                            .errors_mut()
                            .remove(name.as_str());
                    }
                    Err(errors) => {
                        cloned_validation_errors
                            .borrow_mut()
                            .errors_mut()
                            .retain(|key, _| key != &name);
                        for (field_name, error) in errors.errors() {
                            if field_name == &name {
                                cloned_validation_errors
                                    .borrow_mut()
                                    .errors_mut()
                                    .insert(field_name, error.clone());
                            }
                        }
                    }
                }
            }
        })
    };
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
    let on_submit = {
        let cloned_validation_errors = validation_errors.clone();
        let user_id = user_id.clone();
        let passphrase = passphrase.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let form = LoginUserSchema {
                user_id: (*user_id).clone().unwrap_or_default(),
                passphrase: (*passphrase).clone(),
            };
            let validation_errors = cloned_validation_errors.clone();
            let user_id = user_id.clone();
            let passphrase = passphrase.clone();

            match form.validate() {
                Ok(_) => {
                    let _form_data = form.clone();
                    popup_store_dispatch
                        .apply(LoginAction::LoginStarted(form.user_id.clone(), json!({})));
                    let user_id = (*user_id).clone().unwrap_or_default();
                    let passphrase = (*passphrase).clone();
                    login(user_id, passphrase);
                }
                Err(e) => {
                    validation_errors.set(Rc::new(RefCell::new(e)));
                }
            };
        })
    };
    debug!("user_id: {:?}", *user_id);

    html! {
        <>
            <div class="mt-4">
                           <h3 class="relative top-3 text-xl font-medium text-gray-900 dark:text-white">{ "Login" }</h3>
                           <form
                              onsubmit={on_submit}
                                        class="space-y-6 top-5 m-4 pt-5 relative" action="#"

                              >
                              <FormInput label="Email"  name="email" input_type="email"  handle_onchange={handle_user_id_input} errors={&*validation_errors} handle_on_input_blur={validate_input_on_blur.clone()}
                              label_class={
    "block mb-auto text-sm font-medium text-gray-900 dark:text-white"
                              }
        input_class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white" placeholder={"name@company.com"}
                              value={(*user_id).clone()}
                              />
                                  <FormInput label="Passphrase" name="passphrase" input_type="password"  handle_onchange={handle_passphrase_input} errors={&*validation_errors} handle_on_input_blur={validate_input_on_blur.clone()} label_class={"block mb-auto text-sm font-medium text-gray-900 dark:text-white"} input_class={
                                      "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white"
                                  }
                                    value={(*passphrase).clone()}
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
