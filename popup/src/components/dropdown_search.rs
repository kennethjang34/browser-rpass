use std::{cell::RefCell, rc::Rc};

use super::*;
use crate::{dropdown_filter, store::PopupStore, BoolState, BoolStateAction};
use gloo_utils::document;
#[allow(unused_imports)]
use log::*;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yewdux::dispatch::Dispatch;

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct DropdownSearchProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub input_ref: NodeRef,
    #[prop_or_default]
    pub on_select: Option<Callback<Rc<RefCell<DropdownOption>>>>,
    #[prop_or_default]
    pub options: Rc<RefCell<Vec<Rc<RefCell<DropdownOption>>>>>,
    #[prop_or_default]
    pub multiple: bool,
    #[prop_or_default]
    pub force_option: bool,
    #[prop_or_default]
    pub default_input_text: AttrValue,
    #[prop_or_default]
    pub default_selected: Option<Rc<RefCell<DropdownOption>>>,
}

#[function_component(DropdownSearch)]
#[allow(unused_variables)]
pub fn dropdown_search(props: &DropdownSearchProps) -> Html {
    let popup_store_dispatch = Dispatch::<PopupStore>::new();
    let dropdown_open = use_reducer(|| BoolState::new(false));
    let is_default = use_state(|| true);
    let search_text = use_state(|| props.default_input_text.clone());
    let mut options = props.options.borrow().clone();
    let dropdown_options = dropdown_filter((*search_text).as_str(), &mut options, true);
    let search_box_focus = Callback::from({
        let dropdown_open = dropdown_open.clone();
        move |_event: FocusEvent| {
            dropdown_open.dispatch(BoolStateAction::SetAction(true));
        }
    });
    let on_search_box_change = Callback::from({
        let dropdown_open = dropdown_open.clone();
        let search_text = search_text.clone();
        let force_option = props.force_option;
        let dropdown_options = dropdown_options.clone();
        let multiple = props.multiple;
        let option_selected = props.on_select.clone();
        move |event: Event| {
            event.prevent_default();
            if force_option {
                let value = event.target_unchecked_into::<HtmlInputElement>().value();
                if multiple {
                    if let Some(option) = dropdown_options.iter().find(|v| v.borrow().name == value)
                    {
                        if option.borrow().selected {
                            return;
                        }
                        option.borrow_mut().selected = true;
                        if let Some(ref option_selected) = option_selected {
                            option_selected.emit(option.clone());
                        }
                    }
                } else {
                    for option in dropdown_options.iter() {
                        let id = option.borrow().value.clone();
                        let prev_selected = option.borrow().selected;
                        if id == value {
                            option.borrow_mut().selected = true;
                        } else {
                            option.borrow_mut().selected = false;
                        }
                        if prev_selected != option.borrow().selected {
                            if let Some(ref option_selected) = option_selected {
                                option_selected.emit(option.clone());
                            }
                        }
                    }
                }
            }
        }
    });
    let trigger = use_force_update();

    let option_selected = {
        let dropdown_open = dropdown_open.clone();
        let search_text = search_text.clone();
        let multiple = props.multiple;
        let on_select = props.on_select.clone();
        Callback::from(move |option: Rc<RefCell<DropdownOption>>| {
            let selected = !option.borrow().selected;
            {
                option.borrow_mut().selected = selected;
            }
            if !multiple {
                search_text.set(option.borrow().name.clone().into());
                dropdown_open.dispatch(BoolStateAction::SetAction(false));
            } else {
                search_text.set("".into());
                let value = option.borrow().value.clone();
            }
            if let Some(ref on_select) = on_select {
                on_select.emit(option.clone());
            }
            trigger.force_update();
        })
    };
    let on_search_input = Callback::from({
        let search_text = search_text.clone();
        let dropdown_open = dropdown_open.clone();
        let option_forced = props.force_option;
        let multiple = props.multiple;
        move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            search_text.set(value.into());
            dropdown_open.dispatch(BoolStateAction::SetAction(true));
        }
    });
    let search_box_ref = props.input_ref.clone();
    let close_dropdown = {
        let dropdown_open = dropdown_open.clone();
        let search_box_ref = search_box_ref.clone();
        Callback::from(move |event: MouseEvent| {
            event.prevent_default();
            dropdown_open.dispatch(BoolStateAction::SetAction(false));
        })
    };
    let random_id1 = use_memo((), |_| {
        format!("dropdown_search_{}", js_sys::Math::random())
    });
    let random_id2 = use_memo((), |_| {
        format!("dropdown_search_{}", js_sys::Math::random())
    });

    let selected_options = use_state(|| Vec::<Rc<RefCell<DropdownOption>>>::new());
    use_effect_with((random_id1.clone(), random_id2.clone()), {
        let dropdown_open = dropdown_open.clone();
        move |(random_id1, random_id2)| {
            let dropdown_open = dropdown_open.clone();
            let on_click = {
                let random_id1 = (**random_id1).clone();
                let random_id2 = (**random_id2).clone();
                let dropdown_open = dropdown_open.clone();
                Closure::<dyn Fn(_)>::new(move |_event: web_sys::MouseEvent| {
                    if let Some(event_target) = _event.target() {
                        if let Ok(target_element) = event_target.clone().dyn_into::<HtmlElement>() {
                            if target_element.id().contains(&random_id1)
                                || target_element.id().contains(&random_id2)
                            {
                                return;
                            }
                        }
                    }
                    dropdown_open.dispatch(BoolStateAction::SetAction(false));
                })
            };
            document()
                .body()
                .unwrap()
                .add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref())
                .unwrap();
            on_click.forget();
        }
    });

    html! {
        <>
    <div class="w-full md:w-1/2 flex flex-col items-center mx-auto" style="height:fit-content; min-height: 4rem;">
    <div class="w-full h-full">
            <div class="flex flex-col items-center relative" style="height:50%;">
                <div class="w-full">
                    <div class="my-2 p-1 flex border border-gray-200 bg-white rounded" style="height:70%;">
                        <div class="flex flex-auto flex-wrap">
                        if props.multiple
                        {
                            {props.options.borrow().iter().filter_map(
                                |option:&Rc<RefCell<DropdownOption>>| {
                                    if option.borrow().selected {
                                        Some(html!{<Bubble text={option.borrow().name.clone()} cancel_handler={
                                            let option = option.clone();
                                            let  selected_options = selected_options.clone();
                                            let on_select = props.on_select.clone();
                                            Callback::from(move |event:MouseEvent|{
                                                event.prevent_default();
                                                option.borrow_mut().selected = false;
                                                let new_selected_options = selected_options.iter().filter(|x| x.borrow().name != option.borrow().name).map(|x| x.clone()).collect::<Vec<Rc<RefCell<DropdownOption>>>>();
                                                selected_options.set(new_selected_options);
                                                if let Some(ref on_select) = on_select{
                                                    on_select.emit(option.clone());
                                                }

                                            })
                                        } />})

                                    }else{
                                        None
                                    }}
                                ).collect::<Html>()}
                        }
                            <div class="flex-1">
                                <input type="text" id={(*random_id1).clone()}
                                class={classes!(
                                        "bg-transparent p-1 px-2 appearance-none outline-none h-full w-full text-gray-800".to_string())
                                }
                                aria-haspopup="true" aria-expanded="true" value={(*search_text).clone()} oninput={on_search_input.clone()}
                                ref={search_box_ref.clone()}
                                onfocus={search_box_focus.clone()}
                                onchange={on_search_box_change.clone()}
                                autocomplete="off"
                                />
                            </div>
                        </div>
                        if (*dropdown_open).into() && dropdown_options.len() > 0 {
                        <div class="text-gray-300 w-8 py-1 pl-2 pr-1 border-l flex items-center border-gray-200">
                            <button class="cursor-pointer w-6 h-6 text-gray-600 outline-none focus:outline-none" onmousedown={close_dropdown}>
                                <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-chevron-up w-4 h-4">
                                    <polyline points="18 15 12 9 6 15"></polyline>
                                </svg>
                            </button>
                        </div>
                        }
                    </div>
                </div>
                    if (*dropdown_open).into() && dropdown_options.len() > 0 {
                    <div class="w-full" id={(*random_id2).clone()}>
                    <Dropdown
                        options={dropdown_options.clone()}
                        on_select={option_selected.clone()}
                        class="absolute"
                            style="overflow-y:auto; max-height: 10rem;"
                        ></Dropdown>
                    </div>
                    }
            </div>
        </div>
    </div>
    </>
    }
}
