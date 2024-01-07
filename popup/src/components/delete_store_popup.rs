use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::{
    api::extension_api::delete_store,
    store::{DataAction, PopupStore, StoreDataStatus},
};
#[allow(unused_imports)]
use log::*;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

#[derive(Validate, Debug, Default, Clone, Serialize, Deserialize)]
struct LoginSchema {
    store_id: String,
}

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct DeleteStorePopupProps {
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
    #[prop_or_default]
    pub handle_close: Callback<MouseEvent>,
    #[prop_or_default]
    pub input: NodeRef,
}

#[function_component(DeleteStorePopup)]
#[allow(unused_variables)]
pub fn delete_store_popup(props: &DeleteStorePopupProps) -> Html {
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    let input_ref = props.input.clone();
    let validation_errors = use_state(|| Rc::new(RefCell::new(ValidationErrors::new())));
    let is_default = use_state(|| true);
    let current_store = use_selector(|state: &PopupStore| state.persistent_data.store_id.clone());
    let dropdown_options = use_selector({
        let current_store = current_store.clone();
        move |state: &PopupStore| {
            let store_ids = state.store_ids.clone();
            Rc::new(RefCell::new(
                store_ids
                    .iter()
                    .cloned()
                    .map({
                        |store_id| {
                            {
                                Rc::new(RefCell::new(DropdownOption::new(
                                    store_id.clone(),
                                    store_id.clone(),
                                )))
                            }
                        }
                    })
                    .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
            ))
        }
    });

    let selected = use_state(|| Option::<Rc<RefCell<DropdownOption>>>::None);
    let page_loading = use_selector(|state: &PopupStore| state.page_loading);

    let _is_loading = use_selector(|state: &PopupStore| state.page_loading);
    let storage_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let on_submit = Callback::from(|event: SubmitEvent| {
        event.prevent_default();
    });
    let on_button_clicked = {
        let cloned_validation_errors = validation_errors.clone();
        let popup_store_dispatch = popup_store_dispatch.clone();
        let is_default = is_default.clone();
        let input_ref = input_ref.clone();
        let selected = selected.clone();
        Callback::from({
            move |event: MouseEvent| {
                event.prevent_default();
                if let Some(option) = selected.clone().as_ref() {
                    let store_id = option.borrow().value.clone();
                    delete_store(store_id, false);
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
        Callback::from(move |_| {
            dispatch.apply(DataAction::Idle);
        })
    };
    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    let store_dispatch = Dispatch::<PopupStore>::new();
    use_effect_with((store_status.clone(), props.handle_close.clone()), {
        let dispatch = store_dispatch.clone();
        move |(store_status, handle_close): &(Rc<StoreDataStatus>, Callback<MouseEvent>)| {
            if let StoreDataStatus::StoreDeleted(_, _) = **store_status {
                handle_close.emit(MouseEvent::new("click").unwrap());
            }
        }
    });

    html! {
            <div id={props.id.clone()} tabindex="-1" aria-hidden="true" class={
                classes!(String::from("shadow-lg fixed top-0 right-0 left-0 justify-center items-center w-full md:inset-0"), props.class.clone())} style={format!("height:70%; overflow-y: auto; {}",props.style.clone())}>
                <div class="relative w-full h-full">
                    <div class="relative bg-white rounded-lg shadow dark:bg-gray-900 h-full">
                        <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600">
                            <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                            {"Delete Store"}
                            </h3>
                            if let StoreDataStatus::StoreDeletionFailed(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Error} on_close_button_clicked={close_toast.clone()} text={format!("Failed to delete store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            if let StoreDataStatus::StoreDeleted(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Success} on_close_button_clicked={close_toast.clone()} text={format!("Successfully deleted store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            <CloseButton onclick={&props.handle_close}/>
                        </div>
                        <form
                                  onsubmit={on_submit}
                                        class="space-y-1.5 p-2.5 relative max-h-full h-80"
                                            autocomplete="off"
                            style="">
                                        <label for="store-menu" class=
            "block mb-auto text-sm font-medium text-gray-900 dark:text-white">
                                            {"Select store to delete"}
                                        </label>

                                             <DropdownSearch options={(*dropdown_options).clone()}
                                        on_select={on_select.clone()}
                                        input_ref={input_ref.clone()}
                                        force_option=true
                                        multiple=false/>
                                    <button type="button" onclick={on_button_clicked} class="absolute mb-1.5 bottom-0 left-1/2 warning-btn disabled:opacity-75"
                                        style="transform:translateX(-50%); width: calc(100% - 1.25rem);"
                                    >
                                        {"Delete store"}
                                    </button>
    </form>
                    </div>
                </div>
            </div>
        }
}
