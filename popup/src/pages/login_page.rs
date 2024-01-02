use crate::components::StoreSwitcher;
#[allow(unused_imports)]
use log::*;
use yew;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(LoginPage)]
pub fn login_page(_props: &Props) -> Html {
    let input_ref = use_node_ref();
    html! {
        <>

            <div class="mt-14 flex-col">
                <h3 class="text-xl font-medium text-gray-900 dark:text-white" style="text-align: center">{ "Login" }</h3>
            </div>
            <StoreSwitcher class="border-0"/>
    </>
    }
}
