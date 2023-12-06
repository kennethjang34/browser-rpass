mod account_entry;
mod account_entry_list;
mod close_button;
mod create_account_popup;
mod edit_account_popup;
mod error_toast;
mod form_input;
mod loading_indicator;
mod search_input;
pub use account_entry::*;
pub use account_entry_list::*;
pub use close_button::*;
pub use create_account_popup::*;
pub use edit_account_popup::*;
pub use error_toast::*;
pub use form_input::*;
pub use loading_indicator::*;
pub use search_input::*;
use yew::{function_component, html, AttrValue, Classes, Html, Properties};

#[function_component(PlusSign)]
pub fn plus_sign() -> Html {
    html! {<svg class="me-1 -ms-1 w-5 h-5" fill="currentColor" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd"></path></svg>}
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
