use crate::api::extension_api::login;
// use crate::api::user_api::api_login_user;
use crate::components::{form_input::FormInput, loading_button::LoadingButton};
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
    email: String,
    #[validate(
        length(min = 1, message = "Password is required"),
        // length(min = 6, message = "Password must be at least 6 characters")
    )]
    passphrase: String,
}
#[derive(Properties, PartialEq)]
pub struct Props {
    // pub port: Option<Port>,
}

fn get_input_callback(
    name: &'static str,
    cloned_form: UseStateHandle<LoginUserSchema>,
) -> Callback<String> {
    Callback::from(move |value| {
        let mut data = cloned_form.deref().clone();
        match name {
            "email" => data.email = value,
            "passphrase" => data.passphrase = value,
            _ => (),
        }
        cloned_form.set(data);
    })
}

#[function_component(LoginPage)]
pub fn login_page(_props: &Props) -> Html {
    log!("LoginPage");
    let form = use_state(|| LoginUserSchema::default());
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));

    let email_input_ref = NodeRef::default();
    let passphrase_input_ref = NodeRef::default();
    let validate_input_on_blur = {
        let cloned_form = form.clone();
        let cloned_validation_errors = validation_errors.clone();
        Callback::from(move |(name, value): (String, String)| {
            let mut data = cloned_form.deref().clone();
            match name.as_str() {
                "email" => data.email = value,
                "passphrase" => data.passphrase = value,
                _ => (),
            }
            cloned_form.set(data);

            match cloned_form.validate() {
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
        })
    };

    let handle_email_input = get_input_callback("email", form.clone());
    let handle_passphrase_input = get_input_callback("passphrase", form.clone());
    let _is_loading = use_selector(|state: &PopupStore| state.page_loading);
    let (popup_store, popup_store_dispatch) = use_store::<PopupStore>();
    let on_submit = {
        let cloned_form = form.clone();
        let cloned_validation_errors = validation_errors.clone();

        let cloned_email_input_ref = email_input_ref.clone();
        let cloned_passphrase_input_ref = passphrase_input_ref.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();

            let form = cloned_form.clone();
            let validation_errors = cloned_validation_errors.clone();

            let email_input_ref = cloned_email_input_ref.clone();
            let passphrase_input_ref = cloned_passphrase_input_ref.clone();
            match form.validate() {
                Ok(_) => {
                    let _form_data = form.deref().clone();
                    popup_store_dispatch.apply(LoginAction::LoginStarted);
                    let email_input = email_input_ref.cast::<HtmlInputElement>().unwrap();
                    let passphrase_input = passphrase_input_ref.cast::<HtmlInputElement>().unwrap();
                    login(passphrase_input.value());
                    let passphrase_input2 = passphrase_input.clone();
                    email_input.set_value("");
                    passphrase_input2.set_value("");
                }
                Err(e) => {
                    validation_errors.set(Rc::new(RefCell::new(e)));
                }
            };
        })
    };

    html! {
        <>
    <section class="bg-ct-blue-600 min-h-screen grid place-items-center">
      <div class="w-full">
        <h1 class="text-4xl xl:text-6xl text-center font-[600] text-ct-yellow-600 mb-4">
          {"Welcome Back"}
        </h1>
        <h2 class="text-lg text-center mb-4 text-ct-dark-200">
          {"Login to have access"}
        </h2>
          <form
            onsubmit={on_submit}
            class="max-w-md w-full mx-auto overflow-hidden shadow-lg bg-ct-dark-200 rounded-2xl p-8 space-y-5"
          >
            <FormInput label="Email"  name="email" input_type="email" input_ref={email_input_ref} handle_onchange={handle_email_input} errors={&*validation_errors} handle_on_input_blur={validate_input_on_blur.clone()} disabled={true}/>
            <FormInput label="Passphrase" name="passphrase" input_type="password" input_ref={passphrase_input_ref} handle_onchange={handle_passphrase_input} errors={&*validation_errors} handle_on_input_blur={validate_input_on_blur.clone()}/>

            <LoadingButton
              loading={popup_store.page_loading}
              text_color={Some("text-ct-blue-600".to_string())}
            >
              {"Login"}
            </LoadingButton>
          </form>
      </div>
    </section>
    </>
    }
}
