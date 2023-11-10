use std::{cell::RefCell, rc::Rc};

use validator::ValidationErrors;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub input_type: Option<String>,
    pub label: String,
    pub name: String,
    pub input_ref: NodeRef,
    pub handle_onchange: Callback<String>,
    pub handle_on_input_blur: Callback<(String, String)>,
    pub errors: Rc<RefCell<ValidationErrors>>,
    pub label_class: Option<String>,
    pub input_class: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: Option<bool>,
}

#[function_component(FormInput)]
pub fn form_input_component(props: &Props) -> Html {
    let input_type = props
        .input_type
        .clone()
        .unwrap_or_else(|| "text".to_string());
    let val_errors = props.errors.borrow();
    let errors = val_errors.field_errors().clone();
    let empty_errors = vec![];
    let error = match errors.get(props.name.as_str()) {
        Some(error) => error,
        None => &empty_errors,
    };
    let error_message = match error.get(0) {
        Some(message) => message.to_string(),
        None => "".to_string(),
    };

    let handle_onchange = props.handle_onchange.clone();
    let onchange = Callback::from(move |event: Event| {
        let target = event.target().unwrap();
        let value = target.unchecked_into::<HtmlInputElement>().value();
        handle_onchange.emit(value);
    });

    let handle_on_input_blur = props.handle_on_input_blur.clone();
    let on_blur = {
        let cloned_input_name = props.name.clone();
        Callback::from(move |event: FocusEvent| {
            let input_name = cloned_input_name.clone();
            let target = event.target().unwrap();
            let value = target.unchecked_into::<HtmlInputElement>().value();
            handle_on_input_blur.emit((input_name, value));
        })
    };

    html! {
    <div>
        <label html={props.name.clone()}for="email" class={props.label_class.clone().unwrap_or("".to_owned())}>
            {props.label.clone()}
        </label>
      <input
        type={input_type}
        placeholder=""
        class={props.input_class.clone().unwrap_or_else(|| "mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500".to_string())}
        ref={props.input_ref.clone()}
        onchange={onchange}
        onblur={on_blur}
        disabled={props.disabled.unwrap_or(false)}
        />
    <span class="text-red-500 text-xs pt-1 block">
        {error_message}
    </span>
    </div>
    }
}
