use yew::prelude::*;
#[derive(Properties, PartialEq, Clone)]
pub struct Props {
    #[prop_or_default]
    pub class: Classes,
    pub onclick: Option<Callback<MouseEvent>>,
    pub style: Option<String>,
}
#[function_component(CloseButton)]
pub fn close_button(props: &Props) -> Html {
    html! {
                        <button type="button" class={classes!("bg-transparent","dark:hover:bg-gray-600","dark:hover:text-white","w-6","h-6","hover:bg-gray-200","hover:text-gray-900","inline-flex", "items-center","justify-center", "right-2.5", "rounded-lg","text-gray-400","text-sm" , props.class.clone())} style={props.style.clone()} onclick={props.onclick.clone()}>
                           <svg class="w-3 h-3" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 14">
                              <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"/>
                           </svg>
                           <span class="sr-only">{"close modal"}</span>
                        </button>
    }
}
