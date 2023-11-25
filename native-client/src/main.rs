use browser_rpass::{request::*, response::*, types::Resource};
#[allow(warnings)]
use hex::FromHex;
use log::*;
use serde_json::{json, error, Value};

use rpass::{
    crypto::CryptoImpl,
    pass::PasswordStore,
    pass::{self, Error, PasswordEntry, Result},
};
use std::{time::SystemTime, path::{Path, Ancestors}};

use fern::colors::{Color, ColoredLevelConfig};
use serde::Serialize;
use std::io::{self, Read, Write};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread, time,
};
/// The 'pointer' to the current PasswordStore is of this convoluted type.
type PasswordStoreType = Arc<Mutex<Arc<Mutex<PasswordStore>>>>;
/// The list of stores that the user have.
type StoreListType = Arc<Mutex<Vec<Arc<Mutex<PasswordStore>>>>>;
fn _copy(id: &str, store: PasswordStoreType) -> pass::Result<String> {
    let entry = match _get_entries(id, store.clone()) {
        Ok(entries) => entries[0].clone(),
        Err(err) => {
            return Err(err);
        }
    };
    let decrypted = entry.secret(&*store.lock()?.lock()?, None);
    if decrypted.is_ok() {
    } else {
    }
    return decrypted;
}
/// Validates the config for password stores.
/// Returns a list of paths that the new store wizard should be run for
fn _validate_stores_config(settings: &config::Config, home: &Option<PathBuf>) -> Vec<PathBuf> {
    let mut incomplete_stores: Vec<PathBuf> = vec![];
    let stores_res = settings.get("stores");
    if let Ok(stores) = stores_res {
        let stores: HashMap<String, config::Value> = stores;

        for store_name in stores.keys() {
            let store: HashMap<String, config::Value> = stores
                .get(store_name)
                .unwrap()
                .clone()
                .into_table()
                .unwrap();

            let password_store_dir_opt = store.get("path");

            if let Some(p) = password_store_dir_opt {
                let p_path = PathBuf::from(p.clone().into_str().unwrap());
                let gpg_id = p_path.clone().join(".gpg-id");

                if !p_path.exists() || !gpg_id.exists() {
                    incomplete_stores.push(PathBuf::from(p.clone().into_str().unwrap()));
                }
            }
        }
    } else if incomplete_stores.is_empty() && home.is_some() {
        incomplete_stores.push(home.clone().unwrap().join(".password_store"));
    }

    incomplete_stores
}
fn get_stores(config: &config::Config, home: &Option<PathBuf>) -> pass::Result<Vec<PasswordStore>> {
    let mut final_stores: Vec<PasswordStore> = vec![];
    let stores_res = config.get("stores");
    if let Ok(stores) = stores_res {
        let stores: HashMap<String, config::Value> = stores;
        for store_name in stores.keys() {
            let store: HashMap<String, config::Value> = stores
                .get(store_name)
                .unwrap()
                .clone()
                .into_table()
                .unwrap();

            let password_store_dir_opt = store.get("path");
            let valid_signing_keys_opt = store.get("valid_signing_keys");
            if let Some(store_dir) = password_store_dir_opt {
                let password_store_dir = Some(PathBuf::from(store_dir.clone().into_str()?));

                let valid_signing_keys = match valid_signing_keys_opt {
                    Some(k) => match k.clone().into_str() {
                        Err(_) => None,
                        Ok(key) => {
                            if key == "-1" {
                                None
                            } else {
                                Some(key)
                            }
                        }
                    },
                    None => None,
                };
                let style_path_opt = match store.get("style_path") {
                    Some(path) => match path.clone().into_str() {
                        Ok(p) => Some(PathBuf::from(p)),
                        Err(_err) => None,
                    },
                    None => None,
                };

                let pgp_impl = match store.get("pgp") {
                    Some(pgp_str) => CryptoImpl::try_from(pgp_str.clone().into_str()?.as_str()),
                    None => Ok(CryptoImpl::GpgMe),
                }?;

                let own_fingerprint = store.get("own_fingerprint");
                let own_fingerprint = match own_fingerprint {
                    None => None,
                    Some(k) => match k.clone().into_str() {
                        Err(_) => None,
                        Ok(key) => match <[u8; 20]>::from_hex(key) {
                            Err(_) => None,
                            Ok(fp) => Some(fp),
                        },
                    },
                };

                final_stores.push(PasswordStore::new(
                        store_name,
                        &password_store_dir,
                        &valid_signing_keys,
                        home,
                        &style_path_opt,
                        &pgp_impl,
                        &own_fingerprint,
                        )?);
            }
        }
    } else if final_stores.is_empty() && home.is_some() {
        let default_path = home.clone().unwrap().join(".password_store");
        if default_path.exists() {
            final_stores.push(PasswordStore::new(
                    "default",
                    &Some(default_path),
                    &None,
                    home,
                    &None,
                    &CryptoImpl::GpgMe,
                    &None,
                    )?);
        }
    }

    Ok(final_stores)
}
fn main() -> pass::Result<()> {
    let _ = setup_logger();
    trace!("Starting rpass");
    let received_message_res = get_message();
    if received_message_res.is_err() {}
    let received_message = received_message_res.unwrap();
    if let Ok(request) = serde_json::from_value::<RequestEnum>(received_message.clone()) {
        match request {
            RequestEnum::Init(request) => {
                let stores=handle_init_request(request)?;
                thread::sleep(time::Duration::from_millis(200));
                listen_to_native_messaging(stores)
            }
            _ => {
                let err_msg=format!("The first message json must have 'init' as key and initialization values as its value. Received message: {:?}",request);
                Err(Error::GenericDyn(err_msg))
            }
        }
    } else {
        let err_msg=format!("The first message json must have 'init' as key and initialization values as its value. Received message: {:?}",received_message);
        Err(Error::GenericDyn(err_msg))
    }
}
fn get_message() -> Result<serde_json::Value> {
    let mut raw_length = [0; 4];
    if let Err(read_length_res) = io::stdin().read_exact(&mut raw_length) {
        return Err(Error::Io(read_length_res));
    }
    let message_length = u32::from_le_bytes(raw_length);
    let mut message = vec![0; message_length as usize];
    io::stdin()
        .read_exact(&mut message)
        .expect("Failed to read message content");
    let parsed = serde_json::from_slice(message.as_slice());
    if let Err(err) = parsed {
        let error_message = format!("Failed to parse JSON: {:?}", err);
        return Err(Error::GenericDyn(error_message));
    } else {
        return Ok(parsed.unwrap());
    }
}

/// Encode a message for transmission, given its content.
fn encode_message<T: Serialize>(message_content: &T) -> Vec<u8> {
    let encoded_content = serde_json::to_string(message_content).expect("Failed to encode JSON");
    let encoded_length = (encoded_content.len() as u32).to_le_bytes();
    [&encoded_length, encoded_content.as_bytes()].concat()
}

/// Send an encoded message to stdout
fn send_message(encoded_message: &[u8]) {
    io::stdout()
        .write_all(encoded_message)
        .expect("Failed to write to stdout");
    io::stdout().flush().expect("Failed to flush stdout");
}
fn check_passphrase(
    store: &PasswordStoreType,
    user_id: Option<String>,
    passphrase: &str,
    ) -> Result<bool> {
    store
        .lock()
        .unwrap()
        .lock()
        .unwrap()
        .verify_passphrase(user_id, passphrase)
}
fn handle_get_request(request: GetRequest, store: &PasswordStoreType) -> pass::Result<GetResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase") {
            let id = request.id;
            match resource {
                Resource::Username => {
                    let locked_store = store.lock()?;
                    let unlocked_store = &*locked_store.lock()?;
                    let mut store_path = unlocked_store.get_store_path();
                    store_path.push(id.clone());
                    let usernames = std::fs::read_dir(store_path)
                        .unwrap()
                        .map(|entry| {
                            let filename = entry.unwrap().file_name().into_string().unwrap();
                            filename.replace(".gpg", "")
                        })
                    .collect::<Vec<String>>();
                    let get_response = {
                        if usernames.len() > 1 || usernames.len() == 0 {
                            GetResponse {
                                data: serde_json::Value::Null,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                                resource: Resource::Username,
                                meta: None,
                            }
                        } else {
                            let data = serde_json::to_value(usernames[0].clone());
                            {
                                if let Ok(data) = data {
                                    GetResponse {
                                        data,
                                        acknowledgement: acknowledgement.clone(),
                                        status: Status::Success,
                                        resource: Resource::Username,
                                        meta: None,
                                    }
                                } else {
                                    GetResponse {
                                        data: serde_json::Value::Null,
                                        acknowledgement: acknowledgement.clone(),
                                        status: Status::Failure,
                                        resource: Resource::Username,
                                        meta: None,
                                    }
                                }
                            }
                        }
                    };
                    // let json = serde_json::to_string(&get_response).unwrap();
                    // send_message(&encode_message(&json));
                    return Ok(get_response);
                }
                Resource::Password => {
                    let encrypted_password = &search(&store.clone(), &id).unwrap()[0];
                    let locked_store = store.lock()?;
                    let locked_store = &*locked_store.lock()?;
                    let get_response = {
                        if let Ok(password) =
                            &encrypted_password.secret(locked_store, Some(passphrase.clone()))
                            {
                                GetResponse {
                                    data: password.clone().into(),
                                    acknowledgement: acknowledgement.clone(),
                                    status: Status::Success,
                                    resource: Resource::Password,
                                    meta: None,
                                }
                            } else {
                                GetResponse {
                                    data: serde_json::Value::Null,
                                    acknowledgement: acknowledgement.clone(),
                                    status: Status::Failure,
                                    resource: Resource::Password,
                                    meta: None,
                                }
                            }
                    };
                    // let json = serde_json::to_string(&get_response).unwrap();
                    // let encoded = encode_message(&json.to_string());
                    // send_message(&encoded);
                    return Ok(get_response);
                }
                Resource::Account => {
                    let encrypted_password_entry = get_entry(&*store.lock()?.lock()?, &id);
                    let locked_store = store.lock()?;
                    let locked_store = &*locked_store.lock()?;
                    let password_entry = encrypted_password_entry.as_ref().map(
                        |encrypted_password_entry: &PasswordEntry| {
                            let mut json_value =
                                serde_json::to_value(encrypted_password_entry).unwrap();
                            json_value.as_object_mut().unwrap().insert(
                                "password".to_owned(),
                                serde_json::Value::String(
                                    encrypted_password_entry
                                    .secret(locked_store, Some(passphrase.clone()))
                                    .unwrap_or("failed to decrypt password".to_string()),
                                    ),
                                    );
                            json_value
                        },
                        );
                    let get_response = {
                        if let Some(data) = password_entry {
                            GetResponse {
                                data,
                                meta: Some(json!({"id":id})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                            }
                        } else {
                            GetResponse {
                                data: serde_json::Value::Null,
                                meta: Some(json!({"id":id})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                            }
                        }
                    };
                    // let json = serde_json::to_string(&get_response).unwrap();
                    // let encoded = encode_message(&json.to_string());
                    // send_message(&encoded);
                    return Ok(get_response);
                }
                _ => {
                    return Err(pass::Error::from(
                            "resource must be either username, password or account",
                            ));
                }
            };
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}
fn handle_search_request(request: SearchRequest, store: &PasswordStoreType) -> pass::Result<SearchResponse> {
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase").cloned() {
            let resource = request.resource;
            let acknowledgement = request.acknowledgement;
            let query = request.query.unwrap_or("".to_string());
            match resource {
                Resource::Username => {
                    let locked_store = store.lock()?;
                    let unlocked_store = &*locked_store.lock()?;
                    let mut store_path = unlocked_store.get_store_path();
                    store_path.push(query.clone());
                    let usernames = std::fs::read_dir(store_path)
                        .unwrap()
                        .map(|entry| {
                            let filename = entry.unwrap().file_name().into_string().unwrap();
                            filename.replace(".gpg", "")
                        })
                    .collect::<Vec<String>>();
                    let search_response = {
                        if let Ok(data) = serde_json::to_value(usernames.clone()) {
                            SearchResponse {
                                data: data.as_array().unwrap().clone().into(),
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                                meta: Some(json!({"query":query.clone()})),
                                resource,
                            }
                        } else {
                            SearchResponse {
                                data: vec![].into(),
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                                meta: Some(json!({"query":query.clone()})),
                                resource,
                            }
                        }
                    };
                    return Ok(search_response);
                }
                Resource::Account => {
                    let encrypted_password_entries = &search(&store.clone(), &query).unwrap();
                    let locked_store = store.lock()?;
                    let locked_store = &*locked_store.lock()?;
                    let decrypted_password_entries = encrypted_password_entries
                        .iter()
                        .map(|encrypted_password_entry| {
                            let mut json_value =
                                serde_json::to_value(encrypted_password_entry).unwrap();
                            json_value.as_object_mut().unwrap().insert(
                                "password".to_owned(),
                                serde_json::Value::String(
                                    encrypted_password_entry
                                    .secret(locked_store, Some(passphrase.clone()))
                                    .unwrap_or("failed to decrypt password".to_string()),
                                    ),
                                    );
                            json_value
                        })
                    .collect::<Vec<serde_json::Value>>();
                    let search_response = {
                        if let Ok(data) = serde_json::to_value(decrypted_password_entries.clone()) {
                            SearchResponse {
                                data: data.as_array().unwrap().clone().into(),
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                                resource,
                                meta: Some(json!({"query":query.clone()})),
                            }
                        } else {
                            SearchResponse {
                                data: vec![].into(),
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                                resource,
                                meta: Some(json!({"query":query.clone()})),
                            }
                        }
                    };
                    return Ok(search_response);
                }
                _ => {
                    return Err(pass::Error::from(
                            "resource must be either username or account",
                            ));
                }
            };
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}
fn handle_fetch_request(request: FetchRequest, store: &PasswordStoreType) -> pass::Result<FetchResponse> {
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase") {
            let path = request.path.as_ref();
            let resource = request.resource;
            let acknowledgement = request.acknowledgement;
            match resource {
                Resource::Username => {
                    let locked_store = store.lock()?;
                    let unlocked_store = &*locked_store.lock()?;
                    let mut store_path = unlocked_store.get_store_path();
                    store_path.push(path.unwrap().to_owned());
                    let usernames = std::fs::read_dir(store_path)
                        .unwrap()
                        .map(|entry| {
                            let filename = entry.unwrap().file_name().into_string().unwrap();
                            filename.replace(".gpg", "")
                        })
                    .collect::<Vec<String>>();
                    let fetch_response = {
                        if let Ok(data) = serde_json::to_value(usernames.clone()) {
                            FetchResponse {
                                data,
                                meta: Some(json!({"path":path.clone()})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                            }
                        } else {
                            FetchResponse {
                                data: serde_json::Value::Null,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                                resource,
                                meta: None,
                            }
                        }
                    };
                    return Ok(fetch_response);
                }
                Resource::Password => {
                    let encrypted_passwords =
                        &get_entries_with_path(&store.clone(), path.cloned()).unwrap();
                    let locked_store = store.lock()?;
                    let locked_store = &*locked_store.lock()?;
                    let decrypted_passwords = encrypted_passwords
                        .iter()
                        .map(|encrypted_password| {
                            encrypted_password
                                .secret(locked_store, Some(passphrase.clone()))
                                .unwrap_or("failed to decrypt password".to_string())
                        })
                    .collect::<Vec<String>>();
                    let fetch_response = {
                        if let Ok(data) = serde_json::to_value(decrypted_passwords.clone()) {
                            FetchResponse {
                                data: data.as_array().unwrap().clone().into(),
                                meta: Some(json!({"path":path.clone()})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                            }
                        } else {
                            FetchResponse {
                                data: serde_json::Value::Null,
                                meta: Some(json!({"path":path.clone()})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                            }
                        }
                    };
                    return Ok(fetch_response)
                }
                Resource::Account => {
                    let encrypted_password_entries =
                        &get_entries_with_path(&store.clone(), path.cloned()).unwrap_or(vec![]);
                    let locked_store = store.lock()?;
                    let locked_store = &*locked_store.lock()?;
                    let decrypted_password_entries = encrypted_password_entries
                        .iter()
                        .map(|encrypted_password_entry| {
                            let mut json_value =
                                serde_json::to_value(encrypted_password_entry).unwrap();
                            if let Ok(decrypted)=encrypted_password_entry
                                .secret(locked_store, Some(passphrase.clone())){
                                    let decrypted=serde_json::from_str::<serde_json::Value>(&decrypted).unwrap();
                                    merge_json(&mut json_value,&decrypted);
                                    debug!("decrypted password entry: {:?}",decrypted);
                                    debug!("json_value: {:?}",json_value);
                                }
                            json_value
                        })
                    .collect::<Vec<serde_json::Value>>();
                    let fetch_response = {
                        if let Ok(data) = serde_json::to_value(decrypted_password_entries.clone()) {
                            FetchResponse {
                                data: data.as_array().unwrap().clone().into(),
                                meta: Some(json!({"path":path.clone()})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Success,
                            }
                        } else {
                            FetchResponse {
                                data: serde_json::Value::Null,
                                meta: Some(json!({"path":path.clone()})),
                                resource,
                                acknowledgement: acknowledgement.clone(),
                                status: Status::Failure,
                            }
                        }
                    };
                    return Ok(fetch_response);
                }
                _ => {
                    error!("requsted resource: {:?} not supported", resource);
                    return Err(pass::Error::from(
                            "resource must be either username or password",
                            ));
                }
            };
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}
fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

fn handle_edit_request(request: EditRequest, store: &PasswordStoreType) -> pass::Result<EditResponse> {
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase").cloned() {
            let value = request.value;
            let resource = request.resource;
            match resource {
                // Resource::Password => {
                //     let username = request.id;
                //     let path = request.domain.unwrap_or("".to_string());
                //     let value = value.as_str().unwrap_or("");
                //     change_password(
                //         value,
                //         &(path + "/" + &username),
                //         store.clone(),
                //         Some(passphrase),
                //         )
                //         .expect("Failed to change password");
                // Ok(())
                // }
                // Resource::Username => {
                //     let username = request.id;
                //     let path = request.domain.unwrap_or("".to_string());
                //     let value = value.as_str().unwrap_or("");
                //     do_rename_file(
                //         &(path.clone() + "/" + &username),
                //         &(path.clone() + "/" + value),
                //         store.clone(),
                //         Some(passphrase),
                //         )
                //         .expect("Failed to rename file");
                // Ok(())
                // }
                Resource::Account =>{
                    let domain = value.get("domain").map(|d|d.as_str().unwrap().to_owned());
                    let username = value.get("username").map(|d|d.as_str().unwrap().to_owned());
                    let password = value.get("password").map(|d|d.as_str().unwrap().to_owned());
                    let updated_entry=update_entry(
                        &(request.id),
                        domain.clone(),
                        username.clone(),password.clone(),
                        store.clone(),
                        Some(passphrase.clone())
                    )?;
                    debug!("updated entry: {:?}",updated_entry);
                    //for now, we are using file path instead of account id to update the entry.
                    //TODO we need to make use account id instead of file path in the future.
                    // let updated_entry=get_entry(&*store.lock()?.lock()?, &file_path_for_temp_id).unwrap();
                    if let Ok(mut entry_data)=serde_json::from_str(&updated_entry.secret(&*store.lock()?.lock()?, Some(passphrase.clone())).unwrap()){
                        let entry_meta = serde_json::to_value(&updated_entry).unwrap();
                        merge_json(&mut entry_data, &entry_meta);
                        let edit_response = EditResponse {
                            acknowledgement: request.acknowledgement,
                            data:entry_data,
                            status: Status::Success,
                            resource: Resource::Account,
                            id:request.id,
                            meta:None,
                        };
                        Ok(edit_response)
                    }else{
                        Err(pass::Error::from("failed to update entry"))
                    }
                },
                _ => {
                    return Err(pass::Error::from(
                            "resource must be either username or password",
                            ));
                }
            }
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}

fn handle_init_request(request: InitRequest) -> pass::Result<StoreListType> {
    let home_dir = request.config.get("home_dir");
    let home = {
        if home_dir.is_some() {
            Some(PathBuf::from(home_dir.unwrap()))
        } else {
            match std::env::var("HOME") {
                Err(_) => None,
                Ok(home_path) => Some(PathBuf::from(home_path)),
            }
        }
    };
    let store_dir = request.config.get("store_dir").cloned();
    let password_store_dir = {
        if store_dir.is_some() {
            Some(store_dir.unwrap())
        } else {
            match std::env::var("PASSWORD_STORE_DIR") {
                Err(_) => None,
                Ok(password_store_dir) => Some(password_store_dir),
            }
        }
    };
    let password_store_signing_key = request.config.get("password_store_signing_key").cloned();
    let password_store_signing_key = {
        if password_store_signing_key.is_some() {
            Some(password_store_signing_key.unwrap())
        } else {
            match std::env::var("PASSWORD_STORE_SIGNING_KEY") {
                Err(_) => None,
                Ok(password_store_signing_key) => Some(password_store_signing_key),
            }
        }
    };
    let _xdg_data_home = match std::env::var("XDG_DATA_HOME") {
        Err(_) => match &home {
            Some(home_path) => home_path.join(".local"),
            None => {
                return Err(pass::Error::from("No home directory set"));
            }
        },
        Ok(data_home_path) => PathBuf::from(data_home_path),
    };

    let config_res = {
        let xdg_config_home = match std::env::var("XDG_CONFIG_HOME") {
            Err(_) => None,
            Ok(config_home_path) => Some(PathBuf::from(config_home_path)),
        };
        pass::read_config(
            &password_store_dir,
            &password_store_signing_key,
            &home,
            &xdg_config_home,
            )
    };
    if let Err(err) = config_res {
        error!("Failed to read config: {:?}", err);
        return Err(err);
    }
    let (config, config_file_location) = config_res.unwrap();

    let stores = get_stores(&config, &home);
    if let Err(err) = stores {
        error!("{:?}",err);
        return Err(err);
    }

    let stores: StoreListType = Arc::new(Mutex::new(
            stores
            .unwrap()
            .into_iter()
            .map(|s| Arc::new(Mutex::new(s)))
            .collect(),
            ));

    if !config_file_location.exists() && stores.lock()?.len() == 1 {
        let mut config_file_dir = config_file_location.clone();
        config_file_dir.pop();
        if let Err(err) = std::fs::create_dir_all(config_file_dir) {
            error!("{:?}",err);
            return Err(pass::Error::from(err));
        }
        if let Err(err) = pass::save_config(stores.clone(), &config_file_location) {
            error!("{:?}",err);
            return Err(err)
        }
    }
    Ok(stores)
}
fn handle_login_request(request: LoginRequest,stores: &StoreListType) -> pass::Result<PasswordStoreType> {
    let user_id = request.username;
    let passphrase = request.passphrase;
    let store={
        let stores_locked=stores.lock()?;
        let filtered=stores_locked.iter().filter(|s|s.lock().unwrap().get_name()==&user_id).collect::<Vec<_>>();
        if filtered.len()==0{
            return Err(Error::GenericDyn(format!("No store found for username: {}",user_id)));
        }
        filtered[0].clone()
    };
    let store: PasswordStoreType = Arc::new(Mutex::new(store));
    let res = store.lock()?.lock()?.reload_password_list();
    if let Err(err) = res {
        return Err(err);
    }

    // verify that the git config is correct
    if !store.lock()?.lock()?.has_configured_username() {
        Err(Error::GenericDyn(
                "Git user.name and user.name must be configured".to_string(),
                ))?;
    }
    for password in &store.lock()?.lock()?.passwords {
        if password.is_in_git == pass::RepositoryStatus::NotInRepo {
            Err(Error::GenericDyn(
                    format!("Password entry: {:?}  not found in the current store",password).to_string(),
                    ))?;
        }
    }
    let verified = check_passphrase(&store.clone(), Some(user_id), &passphrase);
    // let acknowledgement = request.acknowledgement;
    // let status = {
    if let Ok(verified) = verified {
        if verified {
            // Status::Success
            return Ok(store);
        } else {
            // Status::Failure
            return Err(Error::GenericDyn(
                    "Failed to verify passphrase".to_string(),
                    ));
        }
    } else {
        return Err(Error::GenericDyn(
                "Failed to verify passphrase".to_string()));
    }
}
fn handle_logout_request(request:LogoutRequest,store: &PasswordStoreType)->pass::Result<()>{
    let acknowledgement = request.acknowledgement;
    let status = Status::Success;
    // let json = serde_json::json!({"status": status,"acknowledgement": acknowledgement.unwrap_or("".to_string()),});
    // let json = serde_json::to_string(&json).unwrap();
    // let encoded = encode_message(&json.to_string());
    // send_message(&encoded);
    Ok(())
}
fn handle_create_request(request: CreateRequest, store: &PasswordStoreType) -> pass::Result<CreateResponse> {
    debug!("handle_create_request: {:?}",request);
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase") {
            let username = request.username;
            let domain = request.domain;
            let value = request.value;
            let resource = request.resource;
            let acknowledgement = request.acknowledgement;
            let meta = Some(json!({"path":domain}));
            let data;
            let status;
            match resource {
                Resource::Account => {
                    let (status,data) = match create_entry(
                        Some(username.clone()),
                        value.as_str().map(|s| s.to_owned()),
                        Some(domain.clone() + "/" + &username),
                        store.clone(),
                        None,
                        Some(passphrase.clone()),
                        ){
                        Ok(entry)=>{
                            if let Ok(mut entry_data)=serde_json::from_str(entry.secret(&*store.lock()?.lock()?, Some(passphrase.clone())).unwrap().as_str()){
                                let entry_meta = serde_json::to_value(&entry).unwrap();
                                merge_json(&mut entry_data, &entry_meta);
                                status = Status::Success;
                                data = entry_data;
                                debug!("created password entry: {:?}",data);
                                (status,data)

                            }
                            else{
                                status = Status::Failure;
                                data = serde_json::Value::Null;
                                (status,data)
                            }
                        }
                        Err(err)=>{
                            status = Status::Failure;
                            data = serde_json::Value::Null;
                            error!("Failed to create password entry: {:?}", err);
                            (status,data)
                        }

                    };
                    let create_response: CreateResponse = CreateResponse {
                        acknowledgement,
                        data,
                        meta,
                        resource: Resource::Account,
                        status,
                    };
                    return Ok(create_response);
                }
                _ => {
                    todo!()
                }
            };
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}
fn handle_delete_request(request: DeleteRequest, store: &PasswordStoreType) -> pass::Result<DeleteResponse> {
    if let Some(header) = request.header.clone() {
        if let Some(passphrase) = header.get("passphrase").cloned() {
            let id = request.id.clone();
            debug!("handle_delete_request: {:?}",request);
            let acknowledgement = request.acknowledgement;
            let (status, data) = {
                if let Ok(entry_data) =
                    delete_entry(store.clone(), &(id), Some(passphrase.clone()))
                    {
                        // merge_json(&mut entry_data, &serde_json::to_value(&entry).unwrap());
                        (Status::Success, Some(entry_data))
                    } else {
                        (Status::Failure, None)
                    }
            };
            debug!("deleted password entry: {:?}",data);
            let delete_response = DeleteResponse {
                acknowledgement,
                data: data
                    .map(|data| serde_json::to_value(data).unwrap())
                    .unwrap_or_default(),
                    status,
            };
            Ok(delete_response)
        } else {
            return Err(pass::Error::from(
                    "passphrase must be provided for credential",
                    ));
        }
    } else {
        return Err(pass::Error::from("header must be provided for credential"));
    }
}

fn listen_to_native_messaging(mut stores: StoreListType) -> pass::Result<()> {
    trace!("start listening to native messaging");
    let mut store_opt: Option<PasswordStoreType> = None;
    loop {
        let received_message_res = get_message();
        if received_message_res.is_err() {
            continue;
        }
        let received_message = received_message_res.unwrap();
        if let Ok(request) = serde_json::from_value::<RequestEnum>(received_message.clone()) {
            let request_result={
                if let Some(store)=store_opt.as_ref(){
                    match request.clone() {
                        RequestEnum::Get(request) => {
                            let response= handle_get_request(request.clone(), store);
                            if response.is_ok(){
                                let response=ResponseEnum::GetResponse(response.unwrap());
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)

                            }else{
                                let response=
                                    ResponseEnum::GetResponse(GetResponse{
                                        status:Status::Failure,
                                        acknowledgement:request.acknowledgement.clone(),
                                        data:json!({"error_message": response.unwrap_err(), "request":request}),
                                        resource:Resource::Password,
                                        meta:None,
                                    });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        }
                        RequestEnum::Search(request) => {
                            let response= handle_search_request(request.clone(), store);
                            if response.is_ok(){
                                let response=response.unwrap();
                                let response=ResponseEnum::SearchResponse(response);
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)

                            }else{
                                let response=
                                    ResponseEnum::SearchResponse(SearchResponse{
                                        status:Status::Failure,
                                        acknowledgement:request.acknowledgement.clone(),
                                        data:vec![],
                                        resource:request.resource,
                                        meta:None,
                                    });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Fetch(request) => {
                            let response= handle_fetch_request(request.clone(), store);
                            if response.is_ok(){
                                let response=response.unwrap();
                                let response=ResponseEnum::FetchResponse(response);
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)

                            }else{
                                let response=
                                    ResponseEnum::FetchResponse(FetchResponse{
                                        status:Status::Failure,
                                        acknowledgement:request.acknowledgement.clone(),
                                        data:serde_json::Value::Null,
                                        resource:request.resource,
                                        meta:None,
                                    });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        }
                        RequestEnum::Init(request) => {
                            let response=handle_init_request(request);
                            if response.is_ok(){
                                stores=response.unwrap();
                                let response=ResponseEnum::InitResponse(InitResponse{
                                    status:Status::Success,
                                    acknowledgement:None,
                                    data:serde_json::Value::Null,
                                });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=ResponseEnum::InitResponse(InitResponse{
                                    status:Status::Failure,
                                    acknowledgement:None,
                                    data:json!({"error_message": response.unwrap_err()}),
                                });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Login(request) => {
                            let store_res=handle_login_request(request.clone(),&stores);
                            if store_res.is_ok(){
                                store_opt=Some(store_res.unwrap());
                                let response=
                                    ResponseEnum::LoginResponse(
                                        LoginResponse{
                                            status:Status::Success,
                                            acknowledgement:request.acknowledgement.clone(),
                                            data:serde_json::Value::Null,
                                        });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=
                                    ResponseEnum::LoginResponse(
                                        LoginResponse{
                                            status:Status::Failure,
                                            acknowledgement:request.acknowledgement.clone(),
                                            data:json!({"error_message": store_res.unwrap_err()}),
                                        });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Logout(request) => {
                            let res=handle_logout_request(request.clone(),store);
                            if res.is_ok(){
                                store_opt=None;
                                let response=
                                    ResponseEnum::LogoutResponse(
                                        LogoutResponse{
                                            status:Status::Success,
                                            acknowledgement:request.acknowledgement.clone(),
                                            data:serde_json::Value::Null,
                                        });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let error_response=LogoutResponse{
                                    status:Status::Failure,
                                    acknowledgement:request.acknowledgement.clone(),
                                    data:json!({"error_message": res.unwrap_err()}),
                                };
                                let json = serde_json::to_string(&error_response).unwrap();
                                send_message(&encode_message(&json));
                                Err(ResponseEnum::LogoutResponse(error_response))
                            }
                        },
                        RequestEnum::Create(request) => {
                            let response=handle_create_request(request.clone(), store);
                            if response.is_ok(){
                                let create_response=ResponseEnum::CreateResponse(response.unwrap());
                                let json = serde_json::to_string(&create_response).unwrap();
                                let encoded = encode_message(&json.to_string());
                                send_message(&encoded);
                                Ok(create_response)
                            }else{
                                let response=
                                    ResponseEnum::CreateResponse(
                                        CreateResponse{
                                            status:Status::Failure,
                                            acknowledgement:request.acknowledgement.clone(),
                                            data:json!({"error_message": response.unwrap_err(), "request":request}),
                                            resource:request.resource,
                                            meta:None,
                                        });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Delete(request) => {
                            let response=handle_delete_request(request.clone(), store);
                            if response.is_ok(){
                                let response=ResponseEnum::DeleteResponse(response.unwrap());
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=
                                    ResponseEnum::DeleteResponse(DeleteResponse{
                                        status:Status::Failure,
                                        acknowledgement:request.acknowledgement.clone(),
                                        data:json!({"error_message": response.unwrap_err(), "request":request}),
                                    });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Edit(request) => {
                            let response=handle_edit_request(request.clone(), store);
                            if response.is_ok(){
                                let response=ResponseEnum::EditResponse(response.unwrap());
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=
                                    ResponseEnum::EditResponse(EditResponse{
                                        id:request.id.clone(),
                                        status:Status::Failure,
                                        acknowledgement:request.acknowledgement.clone(),
                                        data:json!({"error_message": response.unwrap_err(), "request":request}),
                                        resource:request.resource,
                                        meta:None,
                                    });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        _ => {
                            Err(
                                ResponseEnum::GenericError(
                                    GenericError{
                                        status:Status::Failure,
                                        acknowledgement:request.get_acknowledgement(),
                                        data:json!({"error_message": "Unknown request", "request":request})
                                    }
                                    )
                               )
                                // Err(pass::Error::Generic("Unknown request"))
                        }
                    }
                }else {
                    match request.clone() {
                        RequestEnum::Init(request) => {
                            let response=handle_init_request(request.clone());
                            if response.is_ok(){
                                stores=response.unwrap();
                                let response=ResponseEnum::InitResponse(InitResponse{
                                    status:Status::Success,
                                    acknowledgement:request.acknowledgement.clone(),
                                    data:serde_json::Value::Null,
                                });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=ResponseEnum::InitResponse(InitResponse{
                                    status:Status::Failure,
                                    acknowledgement:request.acknowledgement.clone(),
                                    data:json!({"error_message": response.unwrap_err()}),
                                });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        RequestEnum::Login(request) => {
                            let store_res=handle_login_request(request.clone(),&stores);
                            if store_res.is_ok(){
                                store_opt=Some(store_res.unwrap());
                                let response=
                                    ResponseEnum::LoginResponse(
                                        LoginResponse{
                                            status:Status::Success,
                                            acknowledgement:request.acknowledgement.clone(),
                                            data:serde_json::Value::Null,
                                        });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Ok(response)
                            }else{
                                let response=
                                    ResponseEnum::LoginResponse(
                                    LoginResponse{
                                    status:Status::Failure,
                                    acknowledgement:request.acknowledgement.clone(),
                                    data:json!({"error_message": store_res.unwrap_err()}),
                                });
                                let json = serde_json::to_string(&response).unwrap();
                                send_message(&encode_message(&json));
                                Err(response)
                            }
                        },
                        _ => {
                            error!("Only login request could be accepted when no store has been set. Request was: {:?}", request);
                            Err(
                                ResponseEnum::GenericError(
                                    GenericError{
                                        status:Status::Failure,
                                        acknowledgement:request.get_acknowledgement(),
                                        data:json!({"error_message": "Only login request could be accepted when no store has been set", "request":request})
                                    }
                                    )
                               )
                        }
                    }
                }
            };
            if let Err(error)=request_result{
                error!("Failed to handle request: {request} {error}", request=request,error=error);
                let json = serde_json::to_string(&error).unwrap();
                send_message(&encode_message(&json));
            }
        } else {
            error!("Failed to parse message: {:?}", received_message);
        }
    }
}

fn update_entry(
        id: &str,
        domain: Option<String>,
        new_name: Option<String>,
        password: Option<String>,
        store: PasswordStoreType,
        passphrase: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    let id=id.to_string();
    if new_name.is_some(){
        update_username(&id, &new_name.unwrap(), store.clone(), passphrase.clone())?;
    }
    if domain.is_some(){
        update_entry_item(&id, "domain", &domain.unwrap(), store.clone(), passphrase.clone())?;
    }
    let res=if let Some(password) = password {
        update_entry_item(
            &id,
            "password",
            &password,
            store.clone(),
            passphrase.clone(),
        )
    }else{
        Ok(None)
    }
    ;
    // Ok(())
    res.map(|_| get_entry(&*store.lock().unwrap().lock().unwrap(), &id).unwrap())
}
fn update_entry_item(id:&str, key:&str, value:&str, store: PasswordStoreType, passphrase: Option<String>) -> pass::Result<Option<String>> {
    let entry=get_entry(&*store.lock().unwrap().lock().unwrap(), &id).unwrap();
    let secret=entry.secret(&*store.lock().unwrap().lock().unwrap(), passphrase.clone()).unwrap();
    if let Ok(mut content)=serde_json::from_str::<Value>(&secret){
        let existing=content.get(key);
        if let Some(existing)=existing{
            if let Some(existing)=existing.as_str().map(|v|{v.to_string()}){
                content.as_object_mut().unwrap().insert(key.to_string(),serde_json::Value::String(value.to_string()));
                let content=serde_json::to_string(&content).unwrap();
                update_entry_content(&id,&content, store, passphrase)?;
                Ok(Some(existing))
            }else{
                Err(pass::Error::GenericDyn(format!("existing entry content is in wrong format. Value is not of String type. Existing value: {:?}",existing.to_string()).to_string()))
            }
        }else{
            content.as_object_mut().unwrap().insert(key.to_string(),serde_json::Value::String(value.to_string()));
            let content=serde_json::to_string(&content).unwrap();
            update_entry_content(&id,&content, store, passphrase)?;
            return Ok(None);
        }
    }else{
        Err(pass::Error::Generic("Failed to parse entry content"))
    }
}
fn update_username(id:&str,new_name: &str,store: PasswordStoreType, passphrase: Option<String>) -> pass::Result<Option<String>> {
    update_entry_item(id, "username", new_name, store, passphrase)
}
fn do_rename_file(
    old_name: &str,
    new_name: &str,
    store: PasswordStoreType,
    passphrase: Option<String>,
    ) -> pass::Result<()> {
    let res = store
        .lock()?
        .lock()?
        .rename_file(old_name, &new_name, passphrase);
    res.map(|_| ())
}

fn _create_password_entry(
    password: Option<String>,
    path: Option<String>,
    store: PasswordStoreType,
    note: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    if password.is_none() {
        return Err(pass::Error::Generic(
                "No password is given. Password must be passed to create_password_entry",
                ));
    }
    let mut password = password.unwrap();
    if password.is_empty() {
        return Err(pass::Error::Generic(
                "Password is empty, not saving anything",
                ));
    }
    if path.is_none() {
        return Err(pass::Error::Generic(
                "No path given. Path must be passed to create_password_entry",
                ));
    }
    let path = path.unwrap();
    if path.is_empty() {
        return Err(pass::Error::Generic("Path is empty, not saving anything"));
    }
    if password.contains("otpauth://") {
        error!("It seems like you are trying to save a TOTP code to the password store. This will reduce your 2FA solution to just 1FA, do you want to proceed?");
    }

    if let Some(note) = note {
        password = format!("{password}\n{note}");
    }
    _new_password_save(path.as_ref(), password.as_ref(), store)
}
fn create_entry(
        username: Option<String>,
        password: Option<String>,
        domain: Option<String>,
        store: PasswordStoreType,
        note: Option<String>,
        passphrase: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    if password.is_none() {
        return Err(pass::Error::Generic(
                "No password is given. Password must be passed to create_password_entry",
                ));
    }
    let password = password.unwrap();
    if password.is_empty() {
        return Err(pass::Error::Generic(
                "Password is empty, not saving anything",
                ));
    }
    if domain.is_none() {
        return Err(pass::Error::Generic(
            "No path given. Path must be passed to create_password_entry",
        ));
    }
    let id = uuid::Uuid::new_v4().to_string();
    // if path.is_empty() {
    //     return Err(pass::Error::Generic("Path is empty, not saving anything"));
    // }
    if password.contains("otpauth://") {
        error!("It seems like you are trying to save a TOTP code to the password store. This will reduce your 2FA solution to just 1FA, do you want to proceed?");
    }
    let content=
        serde_json::to_string(
            &json!({
                "username":username,
                "password":password,
                "domain":domain,
                "note":note,
            })).unwrap();

    create_entry_file(id.as_ref(), content.as_ref(), store, passphrase)
}
fn _new_password_save(
    path: &str,
    password: &str,
    store: PasswordStoreType,
    ) -> pass::Result<PasswordEntry> {
    let entry = store
        .lock()?
        .lock()?
        .new_password_file(path.as_ref(), password.as_ref());
    entry
}
fn create_entry_file(
        id: &str,
        json_string: &str,
        store: PasswordStoreType,
        passphrase: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    let entry = store.lock()?.lock()?.new_password_file_with_passphrase(
        id.as_ref(),
        json_string.as_ref(),
        passphrase,
        );
    entry
}

fn update_entry_content(
        entry_id: &str,
        content: &str,
        store: PasswordStoreType,
        passphrase: Option<String>,
    ) -> pass::Result<()> {
    let password_entry_opt = get_entry(&*store.lock()?.lock()?, entry_id);
    if password_entry_opt.is_none() {
        return Err("No entry file found".into());
    }
    let password_entry = password_entry_opt.unwrap();
    let r = password_entry.update(
        content.to_string(),
        &*store.lock()?.lock()?,
        passphrase,
        );
    if r.is_err() {
        error!("Failed to update password: {:?}", r.as_ref().unwrap_err());
    }
    return r;
}
fn _get_entries(query: &str, store: PasswordStoreType) -> pass::Result<Vec<PasswordEntry>> {
    let entries = pass::search(&*store.lock()?.lock()?, &String::from(query));
    if entries.len() == 0 {
        return Err("No entries found".into());
    }
    Ok(entries)
}
fn get_entries_with_path(
    store: &PasswordStoreType,
    path: Option<String>,
    ) -> pass::Result<Vec<PasswordEntry>> {
    let entries = pass::get_entries_with_path(&*store.lock()?.lock()?, path);
    if entries.len() == 0 {
        return Err("No entries found".into());
    }
    Ok(entries)
}
fn search(store: &PasswordStoreType, query: &str) -> pass::Result<Vec<PasswordEntry>> {
    let first_locked = store.lock()?;
    let locked_store = first_locked.lock()?;
    let passwords = &*locked_store.passwords;
    fn normalized(s: &str) -> String {
        s.to_lowercase()
    }
    fn matches(s: &str, q: &str) -> bool {
        normalized(s).as_str().contains(normalized(q).as_str())
    }
    let matching = passwords.iter().filter(|p| matches(&p.id.to_string(), query));
    let result = matching.cloned().collect();
    Ok(result)
}
pub fn get_entry(store: &PasswordStore, path: &str) -> Option<PasswordEntry> {
    let passwords = &store.passwords;
    fn normalized(s: &str) -> String {
        s.to_lowercase()
    }
    fn matches(s: &str, p: &str) -> bool {
        normalized(s).as_str() == normalized(p).as_str()
    }
    let matching = passwords.iter().find(|p| matches(&p.id.to_string(), path)).cloned();
    return matching;
}
pub fn remove_entry(store: &mut PasswordStore, id: &str) -> Option<PasswordEntry> {
    let id = uuid::Uuid::parse_str(id).unwrap();
    let passwords = &mut store.passwords;
    fn normalized(s: &str) -> String {
        s.to_lowercase()
    }
    fn matches(s: &str, p: &str) -> bool {
        normalized(s).as_str() == normalized(p).as_str()
    }
    if let Some(idx) = passwords
        .iter()
            .position(|p| p.id== id)
            {
                let matching = passwords.remove(idx);
                return Some(matching);
            } else {
                return None;
            }
}
fn delete_entry(
    store: PasswordStoreType,
    id: &str,
    passphrase: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    let password_entry_opt = remove_entry(&mut *store.lock()?.lock()?, id);
    if password_entry_opt.is_none() {
        return Err("No password entry found".into());
    }
    let password_entry = password_entry_opt.unwrap();
    password_entry
        .delete_file_passphrase(&*store.lock()?.lock()?, passphrase)
        .map(|_| password_entry)
}
pub fn setup_logger() -> std::result::Result<(), fern::InitError> {
    let home=std::env::var("HOME").unwrap_or("".to_string());
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::BrightWhite)
        .debug(Color::BrightMagenta)
        .trace(Color::BrightBlack);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                    "{header_color}[{date} {target}][{color_line}{level}{header_color}] {color_line}{message} {footer_color}[{file}:{line_number}]\x1B[0m ",
                    header_color=
                    format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                        ),
                        color_line = 
                        format_args!(
                            "\x1B[{}m",
                            colors_line.get_color(&record.level()).to_fg_str()
                            ),
                            date = humantime::format_rfc3339_seconds(SystemTime::now()),
                            target = record.target(),
                            level = record.level(),
                            message = message,
                            footer_color=
                            format_args!(
                                "\x1B[{}m",
                                colors_line.get_color(&record.level()).to_fg_str()
                                ),
                                file = record.file().unwrap_or("unknown"),
                                line_number = record.line().unwrap_or(0)
                                    ));
        })
    .chain(std::io::stderr())
        .chain(
            fern::Dispatch::new().level(LevelFilter::Warn).chain(
                fern::log_file(format!(
                        "{}/rpass/browser-rpass/native-client/logs/output-{}.log",
                        home,
                        chrono::offset::Local::now()
                        ))?))
        .chain(
            fern::Dispatch::new().level(LevelFilter::Warn).chain(
                std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(format!("{}/rpass/browser-rpass/native-client/logs/output.log",home))?,
                ))
        .apply()?;
    Ok(())
}
