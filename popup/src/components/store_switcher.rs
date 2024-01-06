use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::{
    api::extension_api::login,
    store::{LoginAction, LoginStatus, PopupStore},
};
use browser_rpass::types::StorageStatus;
#[allow(unused_imports)]
use log::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{
    dispatch::Dispatch,
    functional::{use_selector, use_selector_with_deps},
};

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
struct LoginSchema {
    store_id: String,
}

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct StoreSwitcherProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub handle_close: Callback<MouseEvent>,
    #[prop_or_default]
    pub input: NodeRef,
}

#[function_component(StoreSwitcher)]
#[allow(unused_variables)]
pub fn store_switcher(props: &StoreSwitcherProps) -> Html {
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    let input_ref = props.input.clone();
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));
    let is_default = use_state(|| true);
    let current_store_id =
        use_selector(|state: &PopupStore| state.persistent_data.store_id.clone());
    let current_store_active =
        use_selector(|state: &PopupStore| state.persistent_data.store_activated.clone());
    let dropdown_options = use_selector_with_deps(
        {
            move |state: &PopupStore, current_store_id: &Rc<Option<String>>| {
                let store_ids = state.store_ids.clone();
                Rc::new(RefCell::new(
                    store_ids
                        .iter()
                        .cloned()
                        .map(|store_id| {
                            let selected = current_store_id.is_some()
                                && store_id == (**current_store_id).clone().unwrap();
                            let option = Rc::new(RefCell::new(DropdownOption {
                                name: store_id.clone(),
                                value: store_id.clone(),
                                selected,
                            }));
                            option
                        })
                        .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
                ))
            }
        },
        current_store_id.clone(),
    );
    let selected = use_state(|| {
        if current_store_id.is_some() {
            let store_id = (*current_store_id).clone().unwrap();
            dropdown_options.borrow().iter().find_map(|option| {
                if option.borrow().value == store_id {
                    Some(option.clone())
                } else {
                    None
                }
            })
        } else {
            None
        }
    });

    let page_loading = use_selector(|state: &PopupStore| state.page_loading);

    let remember_me = use_selector(|state: &PopupStore| state.persistent_data.remember_me);

    let handle_rememeber_me_input = Callback::from({
        let dispatch = popup_store_dispatch.clone();
        move |event: Event| {
            dispatch.apply(LoginAction::RememberMe(
                event.target_unchecked_into::<HtmlInputElement>().checked(),
            ));
        }
    });
    let handle_is_default_input = Callback::from({
        let is_default = is_default.clone();
        move |event: Event| {
            is_default.set(event.target_unchecked_into::<HtmlInputElement>().checked());
        }
    });
    let _is_loading = use_selector(|state: &PopupStore| state.page_loading);
    let login_status = use_selector(|state: &PopupStore| state.login_status.clone());
    let on_submit = Callback::from(|event: SubmitEvent| {
        event.prevent_default();
    });
    let on_switch = {
        let cloned_validation_errors = validation_errors.clone();
        let popup_store_dispatch = popup_store_dispatch.clone();
        let is_default = is_default.clone();
        let input_ref = input_ref.clone();
        let selected = selected.clone();
        let current_store_id = current_store_id.clone();
        Callback::from({
            move |event: MouseEvent| {
                event.prevent_default();
                if let Some(option) = (*selected).clone() {
                    let store_id = option.borrow().value.clone();
                    let form = LoginSchema {
                        store_id: store_id.clone(),
                    };
                    let validation_errors = cloned_validation_errors.clone();

                    match form.validate() {
                        Ok(_) => {
                            let _form_data = form.clone();
                            if let Some(current_store_id) = &(*current_store_id) {
                                if current_store_id != &store_id {
                                    login(store_id, *is_default, Some(current_store_id.clone()));
                                } else {
                                    login(store_id, *is_default, None);
                                }
                            } else {
                                login(store_id, *is_default, None);
                            }
                        }
                        Err(e) => {
                            validation_errors.set(Rc::new(RefCell::new(e)));
                        }
                    };
                } else {
                    let input = input_ref.cast::<HtmlInputElement>().unwrap();
                    let store_id = input.value();
                    if let Some(current_store_id) = &(*current_store_id) {
                        if current_store_id != &store_id {
                            login(store_id, *is_default, Some(current_store_id.clone()));
                        } else {
                            login(store_id, *is_default, None);
                        }
                    } else {
                        login(store_id, *is_default, None);
                    }
                }
            }
        })
    };
    let on_select = {
        let selected = selected.clone();
        Callback::from(move |option: Rc<RefCell<DropdownOption>>| {
            if option.borrow().selected {
                selected.set(Some(option.clone()));
            } else {
                selected.set(None);
            }
        })
    };
    let close_toast = {
        let dispatch = popup_store_dispatch.clone();
        Callback::from(move |_| match dispatch.get().data.storage_status {
            StorageStatus::Loaded => {
                dispatch.apply(LoginAction::LoggedIn);
            }
            _ => {
                dispatch.apply(LoginAction::LogoutIdle);
            }
        })
    };

    html! {
        <>
                    <div class={classes!(String::from("flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600"),props.class.clone())}>
                    if *login_status == LoginStatus::LoginFailed || *login_status == LoginStatus::LoginError {
                        <Toast toast_type={ToastType::Error} class="absolute right-0 mr-5 my-4" text={"Login Failed"} on_close_button_clicked={close_toast.clone()}/>
                    }
                    if *login_status == LoginStatus::LoginSuccess  {
                        <Toast toast_type={ToastType::Success} class="absolute right-0 mr-5 my-4" text={"Login Success"} on_close_button_clicked={close_toast.clone()}/>
                    }
                    </div>
                           <form
                              onsubmit={on_submit}
                                        class="space-y-1.5 p-2.5 relative max-h-full h-80"

                              >
                                    <label for="store-menu" class=
        "block mb-auto text-sm font-medium text-gray-900 dark:text-white">
                                        {"Store"}
                                    </label>

                                         <DropdownSearch options={(*dropdown_options).clone()}
                                    default_input_text={(*current_store_id).clone().unwrap_or("".to_string())}
                                    force_option=true
                                    on_select={on_select.clone()}
                                    input_ref={input_ref.clone()}
                                    multiple=false/>
                     <div class="flex-col">
                        <div class="flex items-start">
                            <div class="flex items-center h-5">
                                <input id="remember" type="checkbox" checked={*remember_me} class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800" required={false} onchange={handle_rememeber_me_input}/>
                            </div>
                            <label for="remember" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{"Remeber Store ID"}</label>
                        </div>
                        <div class="flex items-start">
                            <div class="flex items-center h-5">
                                <input id="is-default" type="checkbox" checked={*is_default} class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800" required={false} onchange={handle_is_default_input}/>
                            </div>
                            <label for="is-default" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{"For Autofill Suggestions"}</label>
                        </div>
                    </div>
                                                      <button type="button" onclick={on_switch} class="absolute bottom-0 left-1/2 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800 disabled:opacity-25"
                                                      style="transform:translateX(-50%); width: calc(100% - 1.25rem);" disabled={
                                                          selected.is_none() || (current_store_id.is_some()
                                                                                 && (*current_store_active)
                                                                                 && (*selected).clone().unwrap().borrow().value==(*current_store_id).clone().unwrap())
                                                      }
                                                      >{"Switch stores"}</button>

                           </form>
    </>
    }
}
