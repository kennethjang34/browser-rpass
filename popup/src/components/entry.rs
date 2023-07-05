use yew::prelude::*;
use yewdux::prelude::use_store;

use crate::store::Store;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {}

#[function_component(AlertComponent)]
pub fn entry_component(props: &Props) -> Html {
    let (store, dispatch) = use_store::<Store>();

    html! {
    <div>
    </div>
    }
}
