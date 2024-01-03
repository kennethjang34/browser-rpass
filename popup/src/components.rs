mod account_entry;
mod account_entry_list;
mod close_button;
mod create_account_popup;
mod create_store_popup;
mod delete_store_popup;
mod dropdown_search;
mod edit_account_popup;
mod form_input;
mod loading_indicator;
mod multi_select;
mod search_input;
mod simple_popup;
mod store_switcher;
mod toast;
use std::{cell::RefCell, rc::Rc};

pub use account_entry::*;
pub use account_entry_list::*;
pub use close_button::*;
pub use create_account_popup::*;
pub use create_store_popup::*;
pub use delete_store_popup::*;
pub use dropdown_search::*;
pub use edit_account_popup::*;
pub use form_input::*;
pub use loading_indicator::*;
use log::debug;
pub use multi_select::*;
pub use search_input::*;
pub use simple_popup::*;
pub use store_switcher::*;
pub use toast::*;
use web_sys::MouseEvent;
use yew::{
    classes, function_component, html, AttrValue, Callback, Classes, Html, NodeRef, Properties,
};

#[function_component(PlusSign)]
pub fn plus_sign() -> Html {
    html! {<svg class="me-1 -ms-1 w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd"></path></svg>}
}
#[function_component(WarningIcon)]
pub fn warning_icon() -> Html {
    html! {
    <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 text-red-500 bg-red-100 rounded-lg dark:bg-red-800 dark:text-red-200">
    <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
    <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/>
    </svg>
    <span class="sr-only">{"Error icon"}</span>
    </div>}
}
#[function_component(CheckIcon)]
pub fn check_icon() -> Html {
    html! {
                        <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 bg-transparent rounded-lg">
                        <svg enable-background="new 0 0 256 256" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg"><path d="m93.3 161.9 118.9-116.6c5.5-5.4 14.4-5.4 19.8 0l9.9 9.7c5.5 5.4 5.5 14.1 0 19.5l-127.3 124.9-1.3 1.6-9.9 9.7c-2.8 2.7-6.4 4.1-10 4-3.6 0-7.3-1.3-10-4l-9.9-9.7-1.3-1.6-57.9-56.8c-5.5-5.4-5.5-14.1 0-19.5l9.9-9.7c5.5-5.4 14.4-5.4 19.8 0z"/></svg>
                        <span class="sr-only">{"Success icon"}</span>
                        </div>
    }
}
#[function_component(EditIcon)]
pub fn edit_icon() -> Html {
    html! {
     <svg class="me-1 -ms-1 w-5 h-5" fill="currentColor" viewBox="0 0 494.936 494.936" xmlns="http://www.w3.org/2000/svg">
            <path d="M389.844,182.85c-6.743,0-12.21,5.467-12.21,12.21v222.968c0,23.562-19.174,42.735-42.736,42.735H67.157
                    c-23.562,0-42.736-19.174-42.736-42.735V150.285c0-23.562,19.174-42.735,42.736-42.735h267.741c6.743,0,12.21-5.467,12.21-12.21
                    s-5.467-12.21-12.21-12.21H67.157C30.126,83.13,0,113.255,0,150.285v267.743c0,37.029,30.126,67.155,67.157,67.155h267.741
                    c37.03,0,67.156-30.126,67.156-67.155V195.061C402.054,188.318,396.587,182.85,389.844,182.85z"/>
            <path d="M483.876,20.791c-14.72-14.72-38.669-14.714-53.377,0L221.352,229.944c-0.28,0.28-3.434,3.559-4.251,5.396l-28.963,65.069
                    c-2.057,4.619-1.056,10.027,2.521,13.6c2.337,2.336,5.461,3.576,8.639,3.576c1.675,0,3.362-0.346,4.96-1.057l65.07-28.963
                    c1.83-0.815,5.114-3.97,5.396-4.25L483.876,74.169c7.131-7.131,11.06-16.61,11.06-26.692
                    C494.936,37.396,491.007,27.915,483.876,20.791z M466.61,56.897L257.457,266.05c-0.035,0.036-0.055,0.078-0.089,0.107
                    l-33.989,15.131L238.51,247.3c0.03-0.036,0.071-0.055,0.107-0.09L447.765,38.058c5.038-5.039,13.819-5.033,18.846,0.005
                    c2.518,2.51,3.905,5.855,3.905,9.414C470.516,51.036,469.127,54.38,466.61,56.897z"/>
    </svg>}
}

#[function_component(OpenEyeIcon)]
pub fn open_eye_icon() -> Html {
    html! { <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 14">
    <g stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2">
        <path d="M10 10a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z"/>
        <path d="M10 13c4.97 0 9-2.686 9-6s-4.03-6-9-6-9 2.686-9 6 4.03 6 9 6Z"/>
        </g>
        </svg> }
}
#[function_component(ClosedEyeIcon)]
pub fn closed_eye_icon() -> Html {
    html! {
        <svg class="w-6 h-6 text-gray-800 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 18">
            <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1.933 10.909A4.357 4.357 0 0 1 1 9c0-1 4-6 9-6m7.6 3.8A5.068 5.068 0 0 1 19 9c0 1-3 6-9 6-.314 0-.62-.014-.918-.04M2 17 18 1m-5 8a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"/>
        </svg>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct TooltipProps {
    #[prop_or_default]
    pub text: AttrValue,
    #[prop_or_default]
    pub style: AttrValue,
    #[prop_or_default]
    pub class: Classes,
}
#[function_component(Tooltip)]
pub fn tooltip(props: &TooltipProps) -> Html {
    html! {
        <div
            style={&props.style}
            class={props.class.clone()}
            >
            {&props.text}
        </div>
    }
}
#[function_component(ErrorIcon)]
pub fn error_icon() -> Html {
    html! {

    <svg class="flex-shrink-0 inline w-6 h-6 me-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 32 32">
            <g>
            <g id="Error_1_">
                <g id="Error">
                    <circle cx="16" cy="16" id="BG" r="16" style="fill:#D72828;"/><path d="M14.5,25h3v-3h-3V25z M14.5,6v13h3V6H14.5z" id="Exclamatory_x5F_Sign" style="fill:#E6E6E6;"/></g></g></g>
    </svg>}
}

#[derive(Properties, PartialEq, Clone)]
pub struct IconProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
}
#[function_component(MoonIcon)]
pub fn moon_icon(props: &IconProps) -> Html {
    html! {

                            <svg class={classes!("fill-yellow-500", props.class.clone())} style={props.style.clone()} fill="currentColor" viewBox="0 0 20 20">
                                <path
                                    d="M10 2a1 1 0 011 1v1a1 1 0 11-2 0V3a1 1 0 011-1zm4 8a4 4 0 11-8 0 4 4 0 018 0zm-.464 4.95l.707.707a1 1 0 001.414-1.414l-.707-.707a1 1 0 00-1.414 1.414zm2.12-10.607a1 1 0 010 1.414l-.706.707a1 1 0 11-1.414-1.414l.707-.707a1 1 0 011.414 0zM17 11a1 1 0 100-2h-1a1 1 0 100 2h1zm-7 4a1 1 0 011 1v1a1 1 0 11-2 0v-1a1 1 0 011-1zM5.05 6.464A1 1 0 106.465 5.05l-.708-.707a1 1 0 00-1.414 1.414l.707.707zm1.414 8.486l-.707.707a1 1 0 01-1.414-1.414l.707-.707a1 1 0 011.414 1.414zM4 11a1 1 0 100-2H3a1 1 0 000 2h1z"
                                    fill-rule="evenodd" clip-rule="evenodd"></path>
                            </svg>
    }
}
#[function_component(SunIcon)]
pub fn sun_icon(props: &IconProps) -> Html {
    html! {
                            <svg class={classes!("fill-gray-500", props.class.clone())} style={props.style.clone()} fill="currentColor" viewBox="0 0 20 20">
                                <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z"></path>
                            </svg>
    }
}
#[derive(Properties, PartialEq, Clone, Eq, Debug, Hash)]
pub struct DropdownOption {
    pub name: String,
    pub value: String,
    selected: bool,
}

impl DropdownOption {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            selected: false,
        }
    }
    pub fn selected(&self) -> bool {
        self.selected
    }
    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
    pub fn toggle_selected(&mut self) {
        self.selected = !self.selected;
    }
}
#[derive(Properties, PartialEq, Clone, Debug)]
pub struct DropdownProps {
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub node_ref: Option<NodeRef>,
    #[prop_or_default]
    pub on_select: Option<Callback<Rc<RefCell<DropdownOption>>>>,
    #[prop_or_default]
    pub options: Vec<Rc<RefCell<DropdownOption>>>,
}

#[function_component(Dropdown)]
pub fn dropdown(props: &DropdownProps) -> Html {
    let DropdownProps { class, style, .. } = props;
    let on_select = props.on_select.clone();
    html! {
        <div
            class={classes!("dropdown-menu",class.clone())}
            style={style.clone()}
            >
                <ul class={classes!("dropdown-item-list")}>
                { props.options.iter().cloned().map(|option| {
                    let  borrowed= option.borrow();
                    let name = borrowed.name.clone();
                    html! {
                        <li
                            class="dropdown-item"
                            id={option.borrow().value.clone()}
                            onmousedown=
                            {
                                Callback::from(
                                    {
                                        let option = option.clone();
                                        let on_select = on_select.clone();
                                        move |event:MouseEvent| {
                                            event.prevent_default();
                                            if let Some(ref on_select) = on_select {
                                                on_select.emit(option.clone());
                                            }
                                       }
                                    }
                                )
                            }
                            >
                            {name}
                        </li>
                    }
                }).collect::<Html>()
                }
                </ul>
        </div>

    }
}

#[derive(Properties, PartialEq, Clone, Debug)]
pub struct BubbleProps {
    #[prop_or_default]
    pub text: AttrValue,
    #[prop_or_default]
    pub cancel_handler: Callback<MouseEvent>,
}
#[function_component(Bubble)]
pub fn bubble(props: &BubbleProps) -> Html {
    html! {
        <div class="flex justify-center items-center m-1 font-medium py-1 px-2 bg-white rounded-full text-gray-700  border border-black-300 overflow-x-hidden">
            <div class="text-xs font-normal leading-none max-w-full flex-initial">{props.text.clone()}</div>
            <div class="flex flex-auto flex-row-reverse">
                <div>
                    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-x cursor-pointer hover:text-gray-400 rounded-full w-4 h-4 ml-2" onclick={props.cancel_handler.clone()}>
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </div>
            </div>
        </div>
    }
}
