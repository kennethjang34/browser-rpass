use crate::components::CloseButton;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub on_close_button_clicked: Option<Callback<MouseEvent>>,
    #[prop_or_default]
    pub style: String,
    #[prop_or_default]
    pub text: AttrValue,
    #[prop_or_default]
    pub toast_type: Option<ToastType>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum ToastType {
    Success,
    Error,
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Info,
}

#[function_component(Toast)]
pub fn toast(props: &Props) -> Html {
    let icon = if let Some(toast_type) = props.toast_type.clone() {
        match toast_type {
            ToastType::Success => Some("CheckIcon"),
            ToastType::Error => Some("WarningIcon"),
            ToastType::Warning => Some("WarningIcon"),
            ToastType::Info => None,
        }
    } else {
        None
    };
    let (text_color, dark_text_color, bg_color, dark_bg_color) = {
        if let Some(toast_type) = props.toast_type.clone() {
            match toast_type {
                // text-red-500 bg-red-100 dark:bg-red-800 dark:text-red-200
                ToastType::Success => (
                    "text-gray-500",
                    "dark:text-gray-400",
                    "bg-white",
                    "dark:bg-gray-800",
                ),
                ToastType::Info => (
                    "text-gray-500",
                    "dark:text-gray-400",
                    "bg-white",
                    "dark:bg-gray-800",
                ),
                ToastType::Error => (
                    "text-red-500",
                    "dark:text-red-200",
                    "bg-red-100",
                    "dark:bg-red-800",
                ),
                ToastType::Warning => (
                    "text-red-500",
                    "dark:text-red-200",
                    "bg-red-100",
                    "dark:bg-red-800",
                ),
            }
        } else {
            (
                "text-gray-500",
                "dark:text-gray-400",
                "bg-white",
                "dark:bg-gray-800",
            )
        }
    };
    html! {
                    <div class={classes!(String::from("flex items-center max-w-xs p-3 rounded-lg shadow"),text_color,dark_text_color,bg_color,dark_bg_color,props.class.clone())}
                    id={props.id.clone()}
                    style={"width: fit-content; max-width:20rem;".to_string()+&props.style}
                    role="alert">
                        {
                            if let Some(icon) = icon {
                                html!{
                                    <@{icon}></@>
                                }
                            }else{
                                html!{}
                            }
                        }
                        <div class="ms-3 text-sm font-normal me-5" style="text-wrap:wrap; word-break: break-word; overflow-y: auto; max-height: 4rem;">
                        {props.text.clone()}
                        {props.children.clone()}
                        </div>

                        <CloseButton onclick={
                            props.on_close_button_clicked.clone()
                        } class={classes!("inline-flex","items-center","justify-center","ms-auto",text_color,dark_text_color)}/>
                    </div>

    }
}
