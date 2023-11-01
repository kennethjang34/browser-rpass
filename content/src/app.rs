use std::{collections::BTreeMap, rc::Rc};

use browser_rpass::{get_domain_name, types::Account};
use gloo_utils::{document, window};
use log::*;
use sublime_fuzzy::best_match;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::{mrc::Mrc, prelude::*};

use crate::{
    store::ContentScriptStore,
    util::{fetch_accounts, find_password_input_element, find_username_input_element},
};
#[derive(Clone, Debug, PartialEq, Eq, Properties, Default)]
pub struct Props {}
#[function_component]
pub fn App(_props: &Props) -> Html {
    trace!("App started");
    let page_domain =
        use_state(|| get_domain_name(&window().location().href().unwrap_or_default()));
    let username_input = use_state(|| "".to_owned());
    let username_input_element = find_username_input_element();
    let current_focus = use_state(|| None::<web_sys::HtmlInputElement>);
    let password_input = use_state(|| "".to_owned());
    let password_input_element = find_password_input_element();
    let verified = use_selector(|state: &ContentScriptStore| state.verified);
    let account_selector = use_selector(|state: &ContentScriptStore| state.data.accounts.clone());
    let accounts = use_state(|| Rc::new(Vec::<Rc<Account>>::new()));
    use_effect_with_deps(
        {
            let page_domain = page_domain.clone();
            let user_input_element = username_input_element.clone();
            let _password_input = password_input.clone();
            let password_input_element = password_input_element.clone();
            let current_focus = current_focus.clone();
            {
                let username_input_element = username_input_element.clone();
                let username_input = username_input.clone();
                let password_input_element = password_input_element.clone();
                move |_| {
                    let current_address =
                        get_domain_name(&window().location().href().unwrap_or_default());
                    if *page_domain != current_address {
                        page_domain.set(current_address);
                    }
                    if let Some(ref username_input_element) = user_input_element {
                        let on_focus = {
                            let user_input_element = username_input_element.clone();
                            let current_focus = current_focus.clone();
                            Closure::<dyn Fn(_)>::new(move |_event: web_sys::FocusEvent| {
                                current_focus.set(Some(user_input_element.clone()));
                            })
                        };
                        username_input_element
                            .add_event_listener_with_callback(
                                "focus",
                                on_focus.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                        on_focus.forget();
                    }
                    if let Some(ref password_input_element) = password_input_element {
                        let on_focus = {
                            let password_input_element = password_input_element.clone();
                            let current_focus = current_focus.clone();
                            Closure::<dyn Fn(_)>::new(move |_event: web_sys::FocusEvent| {
                                current_focus.set(Some(password_input_element.clone()));
                            })
                        };
                        password_input_element
                            .add_event_listener_with_callback(
                                "focus",
                                on_focus.as_ref().unchecked_ref(),
                            )
                            .unwrap();
                        on_focus.forget();
                    }
                    let on_click = {
                        let current_focus = current_focus.clone();
                        let username_input_element = username_input_element.clone();
                        let password_input_element = password_input_element.clone();
                        Closure::<dyn Fn(_)>::new(move |_event: web_sys::MouseEvent| {
                            let event_target =
                                _event.target().unwrap().dyn_into::<HtmlElement>().unwrap();
                            if event_target.class_name().contains("rpass-suggestion")
                                || (username_input_element.is_some()
                                    && event_target
                                        == username_input_element
                                            .clone()
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap())
                                || (password_input_element.is_some()
                                    && event_target
                                        == password_input_element
                                            .clone()
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap())
                            {
                                // do nothing when click on suggestion list
                            } else {
                                current_focus.set(None);
                            }
                        })
                    };
                    document()
                        .body()
                        .unwrap()
                        .add_event_listener_with_callback(
                            "click",
                            on_click.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    on_click.forget();
                    if let Some(username_input_element) = username_input_element {
                        username_input_element.set_oninput(Some(
                            &Closure::<dyn Fn(_)>::new({
                                let username_input_element = username_input_element.clone();
                                let username_input = username_input.clone();
                                move |_event: web_sys::InputEvent| {
                                    let username = username_input_element.value();
                                    username_input.set(username);
                                }
                            })
                            .into_js_value()
                            .unchecked_into(),
                        ));
                    }
                }
            }
        },
        (),
    );
    use_effect_with_deps(
        {
            let accounts = accounts.clone();
            move |verified: &Rc<bool>| {
                debug!("verified: {:?}", verified);
                if **verified {
                    fetch_accounts(None);
                } else {
                    accounts.set(Rc::new(Vec::<Rc<Account>>::new()));
                }
            }
        },
        verified.clone(),
    );
    use_effect_with_deps(
        {
            let accounts = accounts.clone();
            let verified = verified.clone();
            move |(page_domain, account_selector): &(
                UseStateHandle<String>,
                Rc<Mrc<Vec<Rc<Account>>>>,
            )| {
                if *verified {
                    let account_state = account_selector.clone();
                    let result_vec = account_state
                        .borrow()
                        .iter()
                        .cloned()
                        .filter(|account| {
                            let domain = account.domain.as_ref();
                            let page_address = (**page_domain).clone();
                            domain.unwrap_or(&String::new()) == &page_address
                        })
                        .collect::<Vec<Rc<Account>>>();
                    accounts.set(Rc::new(result_vec));
                }
            }
        },
        (page_domain.clone(), account_selector.clone()),
    );

    if (*current_focus).is_none() {
        html!(<></>)
    } else {
        let target_input_element: HtmlElement =
            (*current_focus).to_owned().unwrap().dyn_into().unwrap();
        let target_rect = target_input_element.get_bounding_client_rect();
        let list_style = {
            let top = target_rect.top() + target_rect.height();
            let left = target_rect.left();
            let width = target_rect.width();
            format!(
                "position:fixed; background-color: #f9f9f9;
            top: {top}px; width:{width}px ;left: {left}px;
                    min-width: 160px; box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
                    z-index: 99;"
            )
        };
        html!(
            <div id={format!("{}-suggestions",target_input_element.id())}
                style={list_style} class="rpass-suggestion">
            {
                {
                    let username_input_element=username_input_element.clone();
                    let password_input_element=password_input_element.clone();
                    let username_input=username_input.clone();
                    let mut result_vec: Vec<Rc<Account>> = vec![];
                    if username_input.is_empty(){
                        result_vec=(**accounts).clone();
                    }else{
                        let mut filtered_with_current_input: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                        let account_data=&**accounts;
                        for account in account_data {
                            let account_id = &account.id;
                            let result = best_match(&*username_input, account_id);
                            if let Some(result) = result {
                                let score = result.score();
                                filtered_with_current_input
                                    .entry(score)
                                    .and_modify(|ls| ls.push(account.clone()))
                                    .or_insert_with(|| vec![account.clone()]);
                            }
                        }
                        for vac in filtered_with_current_input.into_values() {
                            for v in vac {
                                result_vec.push(v);
                            }
                        }
                    }
                    (*result_vec).iter().map(|entry|{
                        let on_suggestion_click = {
                            let current_focus=current_focus.clone();
                            let entry=entry.clone();
                            let username_input_element=username_input_element.clone();
                            let password_input_element=password_input_element.clone();
                            Callback::from(move |_event: web_sys::MouseEvent| {
                                _event.prevent_default();
                                if username_input_element.is_some() {
                                    username_input_element.as_ref().unwrap().set_value(&entry.username);
                                }
                                if password_input_element.is_some() {
                                    password_input_element.as_ref().unwrap().set_value(entry.password.as_ref().unwrap_or(&String::new()));
                                }
                                current_focus.set(None);
                            })
                        };
                        let entry_element=html!(<div style="cursor: pointer; border: 3px solid black;" class="rpass-suggestion" onclick={on_suggestion_click}>{entry.username.clone()}</div>);
                        entry_element
                    }).collect::<Html>()}
            }
            </div>
        )
    }
}