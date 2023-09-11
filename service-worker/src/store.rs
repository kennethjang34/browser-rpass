use gloo_utils::format::JsValueSerdeExt;
use once_cell::sync::OnceCell;
use std::{any::type_name, collections::HashMap, ops::Deref, rc::Rc, sync::Mutex};
use wasm_bindgen::JsValue;
use yewdux::prelude::{AsyncReducer, Reducer};

pub use browser_rpass::{
    request::{RequestEnum, StorageUpdate},
    store::AsyncCallback,
    store::MESSAGE_ACKNOWLEDGEMENTS_NATIVE,
    store::MESSAGE_ACKNOWLEDGEMENTS_POP_UP,
    util::{chrome, Port},
};
use gloo::{console::log, net::http::Request, storage::errors::StorageError};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use yewdux::{
    prelude::{init_listener, Dispatch, Listener},
    storage::Area,
    store::Store,
};
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionAction {
    Login(bool),
    LoginError,
    Logout,
}

lazy_static! {
    pub static ref NATIVE_PORT: Port = chrome.runtime().connect_native("com.rpass");
}
// pub static EXTENSION_PORT: OnceCell<Port> = OnceCell::new();
lazy_static! {
    pub static ref EXTENSION_PORT: Mutex<Option<Port>> = Mutex::new(None);
}
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq)]
pub struct SessionStore {
    pub passphrase: Option<String>,
    pub verified: bool,
    // pub event: Option<SessionEvent>,
}

impl Store for SessionStore {
    fn new() -> Self {
        init_listener(StorageListener);
        let loaded = SessionStore::load().ok().flatten().unwrap_or_default();
        loaded
    }
    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
impl SessionStore {
    fn to_hashmap(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert(
            "passphrase".to_owned(),
            self.passphrase.clone().unwrap_or_default(),
        );
        map.insert("verified".to_owned(), self.verified.to_string());
        map
    }
    // pub fn get_current_event(&self) -> Option<SessionEvent> {
    //     self.event.clone()
    // }
    pub fn load() -> Result<Option<SessionStore>, StorageError> {
        let storage = chrome.storage().session();
        let value = storage.get_string_value_sync(type_name::<SessionStore>());
        match value {
            Ok(value) => {
                if let Some(value) = value {
                    let state = serde_json::from_str(&value);
                    if let Ok(state) = state {
                        Ok(Some(state))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Err(err) => Err(err),
        }
    }
    pub fn save<T: Serialize>(state: &T, area: Area) -> Result<(), StorageError> {
        let storage = chrome.storage().session();
        let value = serde_json::to_string(state)
            .map_err(|serde_error| StorageError::SerdeError(serde_error))?;
        chrome
            .storage()
            .session()
            .set_string_item_sync(type_name::<T>().to_owned(), value);
        Ok(())
    }
}
impl Reducer<SessionStore> for SessionAction {
    fn apply(self, store: Rc<SessionStore>) -> Rc<SessionStore> {
        log!("SessionAction::apply in reducer called");
        let (session_store, session_event) = match self {
            SessionAction::Login(verified) => (
                SessionStore {
                    verified,
                    ..store.deref().clone()
                }
                .into(),
                Some(SessionAction::Login(true)),
            ),
            SessionAction::LoginError => (
                SessionStore {
                    verified: false,
                    passphrase: None,
                    ..store.deref().clone()
                }
                .into(),
                Some(SessionAction::LoginError),
            ),
            SessionAction::Logout => (
                SessionStore {
                    verified: false,
                    passphrase: None,
                    ..store.deref().clone()
                }
                .into(),
                Some(SessionAction::Logout),
            ),
            _ => (
                SessionStore {
                    ..store.deref().clone()
                }
                .into(),
                None,
            ),
        };
        //no broadcasting session event for now
        // broadcast_session_event(session_event);
        session_store
    }
}

fn broadcast_session_action(session_action: SessionAction) {
    if let Some(port) = EXTENSION_PORT.lock().unwrap().as_ref() {
        let storage_update = RequestEnum::create_storage_update_request(
            None,
            serde_json::to_value(session_action).unwrap(),
            None,
            None,
        );
        port.post_message(<JsValue as JsValueSerdeExt>::from_serde(&storage_update).unwrap());
    }
}
struct StorageListener;
impl Listener for StorageListener {
    type Store = SessionStore;

    fn on_change(&mut self, state: Rc<Self::Store>) {
        log!("StorageListener::on_change called");
        if let Err(err) = Self::Store::save(state.as_ref(), Area::Session) {
            println!("Error saving state to storage: {:?}", err);
        } else {
            log!("StorageListener::on_change called, state saved");
        }
    }
}

pub fn set_passphrase(passphrase: Option<String>, dispatch: Dispatch<SessionStore>) {
    dispatch.reduce_mut(move |store| {
        store.passphrase = passphrase.clone();
        if let Some(ref passphrase) = passphrase {
            let passphrase = passphrase.clone();
        }
    })
}
pub async fn set_verified_status_async(verified: bool, dispatch: Dispatch<SessionStore>) {
    dispatch
        .reduce_mut_future(|store| {
            Box::pin(async move {
                log!(
                    "set_verified_status_async, old verified: {:?}, new verified: {:?}",
                    store.verified,
                    verified
                );
                store.verified = verified;
            })
        })
        .await;
}
pub async fn set_passphrase_async(passphrase: Option<String>, dispatch: Dispatch<SessionStore>) {
    dispatch
        .reduce_mut_future(|store| {
            Box::pin(async move {
                store.passphrase = passphrase.clone();
            })
        })
        .await;
}
pub fn set_verified_status(verified: bool, dispatch: Dispatch<SessionStore>) {
    dispatch.reduce_mut(move |store| {
        store.verified = verified;
    })
}
