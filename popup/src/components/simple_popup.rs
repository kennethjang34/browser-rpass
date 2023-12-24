use super::*;
#[allow(unused_imports)]
use log::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SimplePopupProps {
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
#[function_component(SimplePopup)]
pub fn simple_popup(props: &SimplePopupProps) -> yew::Html {
    html! {
        <div id={props.id.clone()} tabindex="-1" aria-hidden="true" class={
            classes!("h-96",String::from("overflow-y-auto overflow-x-hidden shadow-lg fixed top-0 right-0 left-0 justify-center items-center w-full md:inset-0"), props.class.clone())}>
            <div class="relative w-full h-full">
                <div class="relative bg-white rounded-lg shadow dark:bg-gray-900 h-full">
                        <CloseButton onclick={&props.handle_close} class={classes!("right-0", "absolute")}/>
                    {
                        props.children.clone()
                    }
                </div>
            </div>
        </div>
    }
}
