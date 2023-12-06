use crate::components::CloseButton;
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
    pub on_close_button_clicked: Option<Callback<MouseEvent>>,
    pub style: Option<String>,
    #[prop_or_default]
    pub text: AttrValue,
}
#[function_component(ErrorToast)]
pub fn error_toast(props: &Props) -> Html {
    html! {
                    <div class={classes!(String::from("flex items-center max-w-xs p-4 text-gray-500 bg-white rounded-lg shadow dark:text-gray-400 dark:bg-gray-800"),props.class.clone())}
                    style={"width: fit-content;".to_string()+props.style.clone().unwrap_or_default().as_str()}
                    role="alert">
                        <div class="inline-flex items-center justify-center flex-shrink-0 w-8 h-8 text-red-500 bg-red-100 rounded-lg dark:bg-red-800 dark:text-red-200">
                        <svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M10 .5a9.5 9.5 0 1 0 9.5 9.5A9.51 9.51 0 0 0 10 .5Zm3.707 11.793a1 1 0 1 1-1.414 1.414L10 11.414l-2.293 2.293a1 1 0 0 1-1.414-1.414L8.586 10 6.293 7.707a1 1 0 0 1 1.414-1.414L10 8.586l2.293-2.293a1 1 0 0 1 1.414 1.414L11.414 10l2.293 2.293Z"/>
                        </svg>
                        <span class="sr-only">{"Error icon"}</span>
                        </div>
                        <div class="ms-3 text-sm font-normal me-5">{props.text.clone()}</div>
                        <CloseButton onclick={
                            props.on_close_button_clicked.clone()
                        } class={"inline-flex items-center justify-center ms-auto"}/>
                    </div>

    }
}
