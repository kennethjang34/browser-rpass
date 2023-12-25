use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::*;
use crate::{
    api::extension_api::login,
    store::{LoginAction, LoginStatus, PopupStore},
};
#[allow(unused_imports)]
use log::*;
use serde::{Deserialize, Serialize};
use sublime_fuzzy::best_match;
use validator::{Validate, ValidationErrors};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

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
}

#[function_component(StoreSwitcher)]
#[allow(unused_variables)]
pub fn store_switcher(props: &StoreSwitcherProps) -> Html {
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    let store_ids = use_selector(|state: &PopupStore| state.store_ids.clone());
    let dropdown_open = use_state(|| false);
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));
    let is_default = use_state(|| true);
    let store_id_text_input = use_state(|| {
        popup_store_dispatch
            .get()
            .persistent_data
            .store_id
            .clone()
            .unwrap_or_default()
    });
    let handle_store_id_input = Callback::from({
        let store_id_text_input = store_id_text_input.clone();
        move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            store_id_text_input.set(value);
        }
    });
    let dropdown_options = {
        let mut non_matched: Vec<String> = vec![];
        let mut result = vec![];
        store_ids.iter().cloned().for_each(|store_id| {
            let m_res = best_match(&store_id_text_input, store_id.as_str());
            if let Some(m_res) = m_res {
                let score = m_res.score();
                result.push((score, store_id));
            } else {
                non_matched.push(store_id);
            }
        });
        let mut result_vec: Vec<DropdownOption> = vec![];
        result.sort_by(|a, b| a.0.cmp(&b.0));
        for vac in result {
            result_vec.push(DropdownOption {
                name: vac.1.clone(),
                value: vac.1.clone(),
            });
        }
        for v in non_matched {
            result_vec.push(DropdownOption {
                name: v.clone(),
                value: v.clone(),
            });
        }
        result_vec
    };
    let page_loading = use_selector(|state: &PopupStore| state.page_loading);
    let store_id_input_focused = Callback::from({
        let dropdown_open = dropdown_open.clone();
        move |_event: FocusEvent| {
            dropdown_open.set(true);
        }
    });
    let store_id_input_blur = Callback::from({
        let dropdown_open = dropdown_open.clone();
        move |_event: FocusEvent| {
            dropdown_open.set(false);
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
    let handle_is_default_input = Callback::from({
        let is_default = is_default.clone();
        move |event: Event| {
            is_default.set(event.target_unchecked_into::<HtmlInputElement>().checked());
        }
    });
    let _is_loading = use_selector(|state: &PopupStore| state.page_loading);
    let login_status = use_selector(|state: &PopupStore| state.login_status.clone());
    let on_submit = {
        let cloned_validation_errors = validation_errors.clone();
        let popup_store_dispatch = popup_store_dispatch.clone();
        let is_default = is_default.clone();
        let store_id_text_input = store_id_text_input.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let form = LoginSchema {
                store_id: (*store_id_text_input).clone(),
            };
            let validation_errors = cloned_validation_errors.clone();
            let store_id_text_input = store_id_text_input.clone();

            match form.validate() {
                Ok(_) => {
                    let _form_data = form.clone();
                    popup_store_dispatch.apply(LoginAction::LoginStarted(
                        form.store_id.clone(),
                        HashMap::new(),
                    ));
                    let store_id = (*store_id_text_input).clone();
                    login(store_id, *is_default);
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
    let on_dropdown_click = {
        let dropdown_open = dropdown_open.clone();
        Callback::from(move |_e: MouseEvent| {
            dropdown_open.set(!*dropdown_open);
        })
    };
    let option_selected = {
        let dropdown_open = dropdown_open.clone();
        let store_id_text_input = store_id_text_input.clone();
        Callback::from(move |option: DropdownOption| {
            store_id_text_input.set(option.value.clone());
            dropdown_open.set(false);
        })
    };

    html! {
        <>
                    <div class={classes!(String::from("flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600"),props.class.clone())}>
                    if *login_status == LoginStatus::LoginFailed || *login_status == LoginStatus::LoginError {
                        <ErrorToast class="absolute right-0 mr-5 my-4" text={"Login Failed"} on_close_button_clicked={close_login_error}/>
                    }
                    </div>
                           <form
                              onsubmit={on_submit}
                                        class="space-y-1.5 p-2.5 relative max-h-full h-80" action="#"

                              >
                                    <label for="store-menu" class=
        "block mb-auto text-sm font-medium text-gray-900 dark:text-white">
                                        {"Store"}
                                    </label>
                                    <input type="text" id="store-menu" class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block w-full p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white" aria-haspopup="true" aria-expanded="true" aria-labelledby="store-menu-label" value={(*store_id_text_input).clone()} oninput={handle_store_id_input}
                                onfocus={store_id_input_focused}
                                onblur={store_id_input_blur}
                                autocomplete="off"
                                />
                                    if *dropdown_open && !store_ids.is_empty() && !store_id_text_input.is_empty() {
                                      <Dropdown
                                        class="bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-600 dark:border-gray-500 dark:placeholder-gray-400 dark:text-white absolute"
                                        style="transform:translateX(-50%); width:calc(100% - 1.25rem); left:50%; transform: translateX(-50%);"
                                      on_menu_click={on_dropdown_click}
                                      options={
                                      dropdown_options} on_select={option_selected}
                                      ></Dropdown>
                                    }
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
                                                      <button type="submit" class="absolute bottom-0 left-1/2 text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
                                                      style="transform:translateX(-50%); width: calc(100% - 1.25rem);"
                                                      >{"Switch stores"}</button>

                           </form>
    </>
    }
}
