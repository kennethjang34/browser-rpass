use std::{collections::BTreeMap, rc::Rc};

use browser_rpass::types::Account;
use gloo_utils::document;
use log::*;
use sublime_fuzzy::best_match;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{console, HtmlElement};
use yew::prelude::*;
use yewdux::{mrc::Mrc, prelude::*};

use crate::{
    store::ContentScriptStore,
    util::{fetch_accounts, find_password_input_element, find_username_input_element},
};
#[derive(Clone, Debug, PartialEq, Eq, Properties, Default)]
pub struct Props {
    pub address: String,
}
#[function_component]
pub fn App(props: &Props) -> Html {
    trace!("App started");
    let state = Dispatch::<ContentScriptStore>::new().get();
    info!("state: {:?}", state);
    dbg!(&state);

    let username_input = use_state(|| "".to_owned());
    let username_input_element = find_username_input_element();
    let current_focus = use_state(|| None::<web_sys::HtmlInputElement>);
    let password_input = use_state(|| "".to_owned());
    let password_input_element = find_password_input_element();
    let _address = props.address.clone();
    let suggestions = use_state(|| Vec::<(String, String)>::new());
    use_effect_with_deps(
        {
            let _user_input = username_input.clone();
            let user_input_element = username_input_element.clone();
            let _password_input = password_input.clone();
            let password_input_element = password_input_element.clone();
            let current_focus = current_focus.clone();
            {
                let username_input_element = username_input_element.clone();
                let password_input_element = password_input_element.clone();
                let suggestions = suggestions.clone();
                move |_| {
                    let test_entries = vec![
                        ("test_username1".to_owned(), "test_password1".to_owned()),
                        ("test_username2".to_owned(), "test_password2".to_owned()),
                        ("test_username3".to_owned(), "test_password3".to_owned()),
                        ("test_username4".to_owned(), "test_password4".to_owned()),
                    ];
                    suggestions.set(test_entries);
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
                }
            }
        },
        (),
    );
    let path = use_selector(|state: &ContentScriptStore| state.path.clone());
    let loading = use_selector(|state: &ContentScriptStore| state.page_loading.clone());
    let verified = use_selector(|state: &ContentScriptStore| state.verified);
    let account_selector = use_selector(|state: &ContentScriptStore| state.data.accounts.clone());
    let accounts = use_state(|| Rc::new(Vec::<Rc<Account>>::new()));
    let password_input_ref = NodeRef::default();
    let username_input_ref = NodeRef::default();
    let search_string = use_state(|| String::new());
    let search_input_ref = NodeRef::default();
    use_effect_with_deps(
        {
            let _path = path.clone();
            move |verified: &Rc<bool>| {
                if **verified {
                    fetch_accounts(None);
                }
            }
        },
        verified.clone(),
    );
    use_effect_with_deps(
        {
            let accounts = accounts.clone();
            let verified = verified.clone();
            move |(path, account_selector): &(Rc<Option<String>>, Rc<Mrc<Vec<Rc<Account>>>>)| {
                if *verified {
                    let account_state = account_selector.clone();
                    let mut result: BTreeMap<isize, Vec<Rc<Account>>> = BTreeMap::new();
                    let mut non_matched: Vec<Rc<Account>> = vec![];
                    account_state.borrow().iter().cloned().for_each(|account| {
                        let domain = account.domain.as_ref();
                        let path = (**path).clone().unwrap_or("".to_owned());
                        let m_res = best_match(&path, domain.unwrap_or(&String::new()));
                        if let Some(m_res) = m_res {
                            let score = m_res.score();
                            result
                                .entry(score)
                                .and_modify(|ls| ls.push(account.clone()))
                                .or_insert(vec![account.clone()]);
                        } else {
                            non_matched.push(account);
                        }
                    });
                    let mut result_vec: Vec<Rc<Account>> = vec![];
                    for vac in result.values() {
                        for v in vac {
                            result_vec.push((*v).clone());
                        }
                    }
                    for v in non_matched {
                        result_vec.push(v.clone());
                    }
                    accounts.set(Rc::new(result_vec));
                }
            }
        },
        (path.clone(), account_selector.clone()),
    );
    debug!("accounts: {:?}", accounts);

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

                    (*suggestions).iter().map(|entry|{
                        let on_suggestion_click = {
                            let current_focus=current_focus.clone();
                            let entry=entry.clone();
                            let username_input_element=username_input_element.clone();
                            let password_input_element=password_input_element.clone();
                            Callback::from(move |_event: web_sys::MouseEvent| {
                                _event.prevent_default();
                                if username_input_element.is_some() {
                                    username_input_element.as_ref().unwrap().set_value(&entry.0);
                                }
                                if password_input_element.is_some() {
                                    password_input_element.as_ref().unwrap().set_value(&entry.1);
                                }
                                current_focus.set(None);
                            })
                        };
                        let entry_element=html!(<div style="cursor: pointer; border: 3px solid black;" class="rpass-suggestion" onclick={on_suggestion_click}>{entry.0.clone()}</div>);
                        entry_element
                    }).collect::<Html>()}
            }
            </div>
        )
    }
}
