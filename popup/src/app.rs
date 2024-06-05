use crate::{
    pages::home_page::HomePage,
    store::{PopupAction, PopupStore},
};
use browser_rpass::js_binding::extension_api::*;
use gloo_utils::format::JsValueSerdeExt;
use log::*;
use serde_json::{json, Value};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures;
use yew;
use yew::prelude::*;
use yewdux::{functional::use_selector, prelude::Dispatch};

#[function_component]
pub fn App() -> Html {
    trace!("App");
    let activated = use_selector(|state: &PopupStore| state.persistent_data.store_activated);
    use_effect_with(activated.clone(), {
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let cb: Closure<dyn FnMut(JsValue)> = Closure::new(move |tabs: JsValue| {
                    let tabs: Value = <JsValue as JsValueSerdeExt>::into_serde(&tabs).unwrap();
                    let tabs = tabs.as_array().unwrap();
                    let tabs = tabs
                        .into_iter()
                        .map(|tab| {
                            <JsValue as JsValueSerdeExt>::from_serde(tab)
                                .unwrap()
                                .into()
                        })
                        .collect::<Vec<Tab>>();
                    let dispatch = Dispatch::<PopupStore>::new();
                    let tab = tabs.get(0).unwrap();
                    let host_name = url::Url::parse(&tab.url().unwrap())
                        .unwrap()
                        .host_str()
                        .unwrap_or_default()
                        .to_owned();
                    dispatch.apply(PopupAction::DomainSet(Some(host_name)));
                });
                let _ = chrome
                    .tabs()
                    .query(
                        <JsValue as JsValueSerdeExt>::from_serde(
                            &json!({"active":true,"currentWindow":true}),
                        )
                        .unwrap(),
                    )
                    .then(&cb);
                cb.forget()
            });
        }
    });

    html! {
        <>
            <HomePage/>
        </>
    }
}
