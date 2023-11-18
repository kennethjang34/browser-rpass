use crate::{
    pages::home_page::HomePage,
    store::{PopupAction, PopupStore},
};
use browser_rpass::{
    log,
    util::{chrome, Tab},
};
use gloo_utils::format::JsValueSerdeExt;
use serde_json::{json, Value};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures;
use yew;
use yew::prelude::*;
use yewdux::prelude::Dispatch;

#[function_component]
pub fn App() -> Html {
    let state = Dispatch::<PopupStore>::new().get();
    use_effect_with_deps(
        {
            let state = state.clone();
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
                        let tab = tabs.get(0).unwrap();
                        let _tab_id = tab.id();
                        let host_name = url::Url::parse(&tab.url().unwrap())
                            .unwrap()
                            .host_str()
                            .unwrap()
                            .to_owned();
                        let dispatch = Dispatch::<PopupStore>::new();
                        dispatch.apply(PopupAction::PathSet(Some(host_name)));
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
                wasm_bindgen_futures::spawn_local(async move {
                    if state.verified {
                    } else {
                        log!("not verified");
                    }
                });
            }
        },
        state.verified,
    );

    html! {
        <>
            <HomePage/>
        </>
    }
}
