use std::rc::Rc;

use crate::{
    api::extension_api::create_account,
    components::CloseButton,
    components::ErrorToast,
    components::PlusSign,
    store::{DataAction, PopupStore, StoreDataStatus},
};
#[allow(unused_imports)]
use log::*;
use yew;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub domain: Option<String>,
    pub handle_close: Callback<MouseEvent>,
}

#[function_component(CreateAccountPopup)]
pub fn create_account_popup(props: &Props) -> Html {
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
    let domain_input_ref = NodeRef::default();
    let note_input_ref = NodeRef::default();
    let on_create_submit = Callback::from({
        let password_input_ref = password_input_ref.clone();
        let username_input_ref = username_input_ref.clone();
        let domain_input_ref = domain_input_ref.clone();
        let note_input_ref = note_input_ref.clone();
        move |event: SubmitEvent| {
            event.prevent_default();
            let password_input = password_input_ref.cast::<HtmlInputElement>().unwrap();
            let username_input = username_input_ref.cast::<HtmlInputElement>().unwrap();
            let domain_input = domain_input_ref.cast::<HtmlInputElement>().unwrap();
            let note_input = note_input_ref.cast::<HtmlInputElement>().unwrap();
            create_account(
                Some(domain_input.value()),
                Some(username_input.value()),
                Some(password_input.value()),
                Some(note_input.value()),
            );
        }
    });
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let store_dispatch = Dispatch::<PopupStore>::new();
    let close_error = {
        let dispatch = store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };
    use_effect_with_deps(
        {
            let dispatch = store_dispatch.clone();
            move |(store_status, handle_close): &(Rc<StoreDataStatus>, Callback<MouseEvent>)| {
                if **store_status == StoreDataStatus::CreationSuccess {
                    handle_close.emit(MouseEvent::new("click").unwrap());
                    dispatch.apply(DataAction::Idle);
                }
            }
        },
        (store_status.clone(), props.handle_close.clone()),
    );
    html! {

        <div id="create-account-popup" tabindex="-1" aria-hidden="true" class="overflow-y-auto overflow-x-hidden shadow-lg fixed top-0 right-0 left-0 justify-center items-center w-full md:inset-0">
            <div class="relative w-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900">
                    <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {"Create Account"}
                        </h3>
                        if *store_status == StoreDataStatus::CreationFailed{
                            <ErrorToast class="absolute right-0 mr-5 my-4" text={"Creation Failed"} on_close_button_clicked={close_error}/>
                        }
                        <CloseButton onclick={&props.handle_close}/>
                    </div>
                    <form onsubmit={on_create_submit} class="p-4 md:p-5">
                        <div class="grid gap-4 mb-4 grid-cols-2">
                            <div class="col-span-2">
                                <label for="user-id" class="form-label">{"User ID"}</label>
                                <input type="text" name="user-id" id="user-id" class="form-input" placeholder="User ID" required={true} ref={username_input_ref}/>
                            </div>
                            <div class="col-span-2 sm:col-span-1">
                                <label for="password" class="form-label">{ "Password" }</label>
                                <input type="password" name="password" id="password" class="form-input" placeholder="Password" required={true} ref={password_input_ref}/>
                            </div>
                            <div class="col-span-2 sm:col-span-1">
                                <label for="url" class="form-label">{ "Domain" }</label>
                                <input type="text" name="url" id="url" class="form-input" placeholder="URL" required={true} value={props.domain.clone().unwrap()} ref={domain_input_ref}/>
                            </div>
                            <div class="col-span-2 sm:col-span-1">
                                <label for="note" class="block mb-2 text-sm font-medium text-gray-900 dark:text-white">{ "Note" }</label>
                                <textarea type="text" name="note" id="note" class="form-input" placeholder="Note" required={false}  ref={note_input_ref}/>
                            </div>
                        </div>
                        <button type="submit" class="accent-btn">
                        <PlusSign/>
                            {"Create"}
                        </button>
                    </form>
                </div>
            </div>
        </div>
    }
}
