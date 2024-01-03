use crate::{
    api::extension_api::create_store,
    store::{DataAction, PopupStore, StoreDataStatus},
    BoolState, BoolStateAction,
};

use super::*;
#[allow(unused_imports)]
use log::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::{dispatch::Dispatch, functional::use_selector};

#[derive(Properties, PartialEq)]
pub struct CreateStorePopupProps {
    pub handle_close: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
}
#[function_component(CreateStorePopup)]
pub fn create_store_popup(props: &CreateStorePopupProps) -> yew::Html {
    let store_dispatch = Dispatch::<PopupStore>::new();
    let store_name = use_state(|| {
        if store_dispatch.get().store_ids.len() > 0 {
            String::from("")
        } else {
            "default".to_string()
        }
    });
    let keys = use_selector(|state: &PopupStore| state.keys.clone());
    let recipients = use_state(|| {
        Rc::new(RefCell::new(
            keys.iter()
                .cloned()
                .map(|key| {
                    Rc::new(RefCell::new(DropdownOption::new(
                        format!(
                            "{} <{}> <{}>",
                            key.name.clone().unwrap_or_default(),
                            key.email.clone().unwrap_or_default(),
                            key.id.clone(),
                        ),
                        key.id.clone(),
                    )))
                })
                .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
        ))
    });
    let valid_recipient_signers = use_state(|| {
        Rc::new(RefCell::new(
            keys.iter()
                .cloned()
                .filter_map(|key| {
                    if key.has_secret {
                        Some(Rc::new(RefCell::new(DropdownOption::new(
                            format!(
                                "{} <{}> <{}>",
                                key.name.clone().unwrap_or_default(),
                                key.email.clone().unwrap_or_default(),
                                key.id.clone(),
                            ),
                            key.id.clone(),
                        ))))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
        ))
    });
    let owned_key_options = use_state(|| {
        Rc::new(RefCell::new(
            keys.iter()
                .cloned()
                .filter_map(|key| {
                    if key.has_secret {
                        Some(Rc::new(RefCell::new(DropdownOption::new(
                            format!(
                                "{} <{}> <{}>",
                                key.name.clone().unwrap_or_default(),
                                key.email.clone().unwrap_or_default(),
                                key.id.clone(),
                            ),
                            key.id.clone(),
                        ))))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
        ))
    });

    let store_ids = use_selector(|state: &PopupStore| {
        let store_ids = &state.store_ids;
        RefCell::new(
            store_ids
                .iter()
                .map(|store_id| {
                    Rc::new(RefCell::new(DropdownOption::new(
                        store_id.clone(),
                        store_id.clone(),
                    )))
                })
                .collect::<Vec<Rc<RefCell<DropdownOption>>>>(),
        )
    });
    let parent_store = use_mut_ref(|| Option::<String>::None);
    let on_parent_select = {
        let parent_dir = parent_store.clone();
        Callback::from(move |option: Rc<RefCell<DropdownOption>>| {
            parent_dir.replace(Some(option.borrow().value.clone()));
        })
    };
    let commit_signer_name = use_mut_ref(|| Option::<String>::None);
    let commit_signer_id = use_mut_ref(|| Option::<String>::None);
    let recipient_signer_name = use_mut_ref(|| Option::<String>::None);
    let recipient_signer_id = use_mut_ref(|| Option::<String>::None);
    let on_commit_signer_selected = {
        let commit_signer_name = commit_signer_name.clone();
        let commit_signer_id = commit_signer_id.clone();
        Callback::from(move |option: Rc<RefCell<DropdownOption>>| {
            commit_signer_name.replace(Some(option.borrow().name.clone()));
            commit_signer_id.replace(Some(option.borrow().value.clone()));
        })
    };
    let on_recipient_signer = {
        let recipient_signer_name = recipient_signer_name.clone();
        let recipient_signer_id = recipient_signer_id.clone();
        Callback::from(move |option: Rc<RefCell<DropdownOption>>| {
            recipient_signer_name.replace(Some(option.borrow().name.clone()));
            recipient_signer_id.replace(Some(option.borrow().value.clone()));
        })
    };
    let on_create_submit = Callback::from({
        let store_name_input = store_name.clone();
        let valid_gpg_signers = valid_recipient_signers.clone();
        let commit_signer = commit_signer_id.clone();
        let recipients = recipients.clone();
        move |event: SubmitEvent| {
            let selected_recipients = (*recipients)
                .borrow()
                .iter()
                .filter_map(|s| {
                    if s.borrow().selected {
                        Some(s.borrow().value.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            let valid_signers = (*valid_gpg_signers)
                .borrow()
                .iter()
                .filter_map(|s| {
                    if s.borrow().selected {
                        Some(s.borrow().value.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();
            event.prevent_default();
            create_store(
                (*store_name_input).clone(),
                (*parent_store).borrow().clone(),
                selected_recipients,
                (valid_signers).clone(),
                true,
                (*commit_signer).borrow().clone(),
            );
        }
    });
    let on_store_name_change = {
        let store_name = store_name.clone();
        Callback::from(move |event: InputEvent| {
            event.prevent_default();
            store_name.set(
                event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value(),
            );
        })
    };
    let is_substore = use_reducer(|| BoolState::new(false));
    let configure_valid_signerse = use_reducer(|| BoolState::new(false));
    let on_is_substore_change = {
        let is_substore = is_substore.clone();
        Callback::from(move |event: Event| {
            event.prevent_default();
            is_substore.dispatch(BoolStateAction::ToggleAction);
        })
    };
    let on_configure_valid_signers = {
        let configure_valid_signerse = configure_valid_signerse.clone();
        Callback::from(move |event: Event| {
            event.prevent_default();
            configure_valid_signerse.dispatch(BoolStateAction::ToggleAction);
        })
    };

    let close_toast = {
        let dispatch = store_dispatch.clone();
        Callback::from(move |_| dispatch.apply(DataAction::Idle))
    };

    let store_status = use_selector(|state: &PopupStore| state.data_status.clone());
    use_effect_with((store_status.clone(), props.handle_close.clone()), {
        move |(store_status, handle_close): &(Rc<StoreDataStatus>, Callback<MouseEvent>)| {
            if let StoreDataStatus::StoreCreated(_, _) = **store_status {
                handle_close.emit(MouseEvent::new("click").unwrap());
            }
        }
    });
    html! {
        <div id={props.id.clone()} tabindex="-1" aria-hidden="true" class={
            classes!(String::from("shadow-lg fixed top-0 right-0 left-0 justify-center items-center w-full md:inset-0"), props.class.clone())} style="height:100%; overflow-y: auto; z-index:1000;">
            <div class="relative w-full h-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900 h-full">
                    <div class="flex items-center justify-between p-4 md:p-5 border-b rounded-t dark:border-gray-600" style="height: 8%;">
                        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                        {"Create Store"}
                        </h3>
                            if let StoreDataStatus::StoreCreationFailed(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Error} on_close_button_clicked={close_toast.clone()} text={format!("Failed to create store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                            if let StoreDataStatus::StoreCreated(_,ref store_id)=*store_status{
                                <Toast toast_type={ToastType::Success} on_close_button_clicked={close_toast.clone()} text={format!("Successfully created store: {store_id}")} class="absolute right-0 top-5 z-10"/>
                            }
                        <CloseButton onclick={&props.handle_close}/>
                    </div>
                    <form onsubmit={on_create_submit} class="p-4 md:p-5" style="height:90%; overflow-y:auto;">
                        <div class="grid grid-cols-2 flex">
                            <div class="col-span-2" >
                                <label for="store-name" class="from-label">{"Store Name"}</label>
                                <input type="text" name="store-name" id="store-name" class="form-input"
                                placeholder="Store Name" required={true} value={(*store_name).clone()} oninput={on_store_name_change.clone()}/>
                                <div class="flex items-start">
                                if store_ids.borrow().len()>0{
                                    <div class="flex items-center h-5">
                                        <input type="checkbox" name="is-substore" id="is-substore" class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800"  checked={is_substore.value} onchange={on_is_substore_change.clone()}/>
                                    </div>
                                    <label for="is-substore" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{"is substore?"}</label>
                                }
                                    <div class="flex items-center h-5">
                                        <input type="checkbox" name="configure-valid-signers" id="configure-valid-signers" class="w-4 h-4 border border-gray-300 rounded bg-gray-50 focus:ring-3 focus:ring-blue-300 dark:bg-gray-600 dark:border-gray-500 dark:focus:ring-blue-600 dark:ring-offset-gray-800 dark:focus:ring-offset-gray-800" checked={(*configure_valid_signerse).into()} onchange={on_configure_valid_signers.clone()}/>
                                    </div>
                                    <label for="configure-valid-signers" class="ml-2 text-sm font-medium text-gray-900 dark:text-gray-300">{"configure valid signers"}</label>
                                </div>
                            </div>
                            if (*is_substore).into(){
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="parent-store" class="form-label">{"Parent store"}</label>
                                    <DropdownSearch options={(store_ids).clone()} on_select={on_parent_select} multiple=false/>
                                </div>
                            }
                            if (*configure_valid_signerse).into(){
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="commit-signer" class="form-label">{"Valid signer list"}</label>
                                    <DropdownSearch options={(*valid_recipient_signers).clone()} on_select={on_recipient_signer} multiple=true/>
                                </div>
                            }
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="commit-signer" class="form-label">{"Commit signer"}</label>
                                    <DropdownSearch options={(*owned_key_options).clone()} on_select={on_commit_signer_selected} multiple=false/>
                                </div>
                                <div class="col-span-2 sm:col-span-1">
                                    <label for="store-recipients" class="form-label">{"Store Recipients"}</label>
                                    <DropdownSearch options={(*recipients).clone()} multiple=true/>
                                </div>
                                <div class="col-span-1 sm:col-span-1">
                                    <button type="submit" class="accent-btn p-1.5" style="">
                                    <PlusSign/>
                                        {"Create"}
                                    </button>
                                </div>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    }
}
