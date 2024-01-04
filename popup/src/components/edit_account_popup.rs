use std::rc::Rc;

use crate::{
    api::extension_api::edit_account,
    components::*,
    store::{DataAction, PopupStore, StoreDataStatus},
};
use browser_rpass::types::Account;
#[allow(unused_imports)]
use log::*;
use secrecy::ExposeSecret;
use wasm_bindgen::JsCast;
use yew;

use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub account: Rc<Account>,
    pub store_id: String,
    pub handle_close: Callback<MouseEvent>,
}

#[function_component(EditAccountPopup)]
pub fn edit_account_popup(props: &Props) -> Html {
    let reveal_password = use_state(|| false);
    let account = props.account.clone();
    let password_input = use_state(|| {
        account
            .get_password()
            .map(|p| p.expose_secret().clone())
            .unwrap_or_default()
    });
    let username_input = use_state(|| account.username.clone());
    let note_input = use_state(|| account.note.clone().unwrap_or_default());
    let domain_input = use_state(|| account.domain.clone().unwrap());
    let on_edit_submit = Callback::from({
        let account = account.clone();
        let password_input = password_input.clone();
        let username_input = username_input.clone();
        let note_input = note_input.clone();
        let domain_input = domain_input.clone();
        let store_id = props.store_id.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let domain = Some((*domain_input).clone());
            let username = Some((*username_input).clone());
            let password = Some((*password_input).clone());
            let note = Some((*note_input).clone());
            edit_account(
                account.id.clone(),
                domain,
                username,
                password,
                note,
                Some(store_id.clone()),
            );
        }
    });
    let on_reveal = {
        let reveal_password = reveal_password.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            let value = !*reveal_password;
            reveal_password.set(value);
        })
    };
    let on_password_input = {
        let password_input = password_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            password_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let on_note_input = {
        let note_input = note_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            note_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlTextAreaElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let on_username_input = {
        let username_input = username_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            username_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let on_domain_input = {
        let domain_input = domain_input.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            domain_input.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let store_dispatch = Dispatch::<PopupStore>::new();
    use_effect_with((store_status.clone(), props.handle_close.clone()), {
        let store_dispatch = store_dispatch.clone();
        move |(store_status, handle_close): &(Rc<StoreDataStatus>, Callback<MouseEvent>)| {
            if **store_status == StoreDataStatus::EditionSuccess {
                handle_close.emit(MouseEvent::new("click").unwrap());
                store_dispatch.apply(DataAction::Idle);
            }
        }
    });
    let close_toast = {
        let dispatch = store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };
    let password_input_component = |revealed: bool| -> Html {
        let (input_type, eye_tooltip_text, eye_icon) = if revealed {
            ("text", "click to hide password", html! {<ClosedEyeIcon/>})
        } else {
            (
                "password",
                "click to reveal password",
                html! {<OpenEyeIcon/>},
            )
        };
        html! {
            <div class="col-span-2 sm:col-span-1">
                <label for="password" class="form-label">{ "Password" }</label>
                <div class="relative">
                    <input type={input_type} name="password" id="password" class="form-input" placeholder="Password" required={true} value={(*password_input).clone()} oninput={on_password_input.clone()} />
                    <span onclick={on_reveal.clone()} class="absolute cursor-pointer right-2 top-1/2 peer" style="transform: translateY(-50%);">
                        {eye_icon}
                    </span>
                    <Tooltip text={eye_tooltip_text.to_string()}
                    class="tooltip fixed"
                    style={format!("margin:-0.5rem;transform:translate(-100%,-100%);
                                   top:{top};left:{left};",top="81%",left="100%")}/>
                </div>
            </div>
        }
    };
    html! {
        <div id="edit-account-popup" tabindex="-1" aria-hidden="true" class="overflow-y-auto overflow-x-hidden shadow-lg fixed top-0 right-0 left-0 z-50 justify-center items-center w-full md:inset-0  max-h-full" >
            <div class="relative w-full max-h-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900">
                    <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {"Edit Account"}
                        </h3>
                        if *store_status == StoreDataStatus::EditionFailed{
                            <Toast
                                text={"Edition Failed"}
                                on_close_button_clicked={close_toast.clone()}
                                class="absolute right-0 mr-5 my-4"
                                toast_type={ToastType::Error}
                            />
                        }
                        if *store_status == StoreDataStatus::EditionSuccess{
                            <Toast
                                text={"Edition Success"}
                                on_close_button_clicked={close_toast.clone()}
                                class="absolute right-0 mr-5 my-4"
                                toast_type={ToastType::Success}
                            />
                        }
                        <CloseButton onclick={&props.handle_close}/>
                    </div>
                    <form onsubmit={on_edit_submit} class="p-4 md:p-5">
                        <div class="grid gap-4 mb-4 grid-cols-2">
                            <div class="col-span-2">
                                <label for="username" class="from-label">{"Username"}</label>
                                <input type="text" name="username" id="username" class="form-input"
                                placeholder="User ID" required={true} value={(*username_input).clone()} oninput={on_username_input.clone()}/>
                            </div>
                            {password_input_component(*reveal_password)}
                            <div class="col-span-2 sm:col-span-1">
                                <label for="url" class="form-label">{ "Domain/URL" }</label>
                                <input type="text" name="url" id="url" class="form-input" placeholder="URL" required={true} value={(*domain_input).clone()}  oninput={on_domain_input.clone()}/>
                            </div>
                            <div class="col-span-2">
                                <label for="note" class="form-label">{"Note"}</label>
                                <textarea  name="note" id="note" class="form-input" placeholder="Note" required={false} value={(*note_input).clone()} oninput={on_note_input.clone()}/>
                            </div>
                        </div>
                        <button type="submit" class="accent-btn">
                            <EditIcon/>
                            {"Update Account"}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
