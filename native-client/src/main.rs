use browser_rpass::{request::*, response::*, types::Resource};
#[allow(warnings)]
use hex::FromHex;
use log::*;
use serde_json::json;

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
fn handle_get_request(request: GetRequest, store: &PasswordStoreType) -> pass::Result<()> {
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
                    let json = serde_json::to_string(&get_response).unwrap();
                    send_message(&encode_message(&json));
                    return Ok(());
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
                    let json = serde_json::to_string(&get_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
                    let json = serde_json::to_string(&get_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
fn handle_search_request(request: SearchRequest, store: &PasswordStoreType) -> pass::Result<()> {
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
                    let json = serde_json::to_string(&search_response).unwrap();
                    send_message(&encode_message(&json));
                    return Ok(());
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
                    let json = serde_json::to_string(&search_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
fn handle_fetch_request(request: FetchRequest, store: &PasswordStoreType) -> pass::Result<()> {
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
                    let json = serde_json::to_string(&fetch_response).unwrap();
                    send_message(&encode_message(&json));
                    return Ok(());
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
                    let json = serde_json::to_string(&fetch_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
                    let json = serde_json::to_string(&fetch_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
fn handle_edit_request(request: EditRequest, store: &PasswordStoreType) -> pass::Result<()> {
    info!("Handling edit request: {:?}", request);
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase").cloned() {
            let value = request.value;
            let resource = request.resource;
            match resource {
                Resource::Password => {
                    let username = request.id;
                    let path = request.domain.unwrap_or("".to_string());
                    let value = value.as_str().unwrap_or("");
                    change_password(
                        value,
                        &(path + "/" + &username),
                        store.clone(),
                        Some(passphrase),
                        )
                        .expect("Failed to change password");
                    Ok(())
                }
                Resource::Username => {
                    let username = request.id;
                    let path = request.domain.unwrap_or("".to_string());
                    let value = value.as_str().unwrap_or("");
                    do_rename_file(
                        &(path.clone() + "/" + &username),
                        &(path.clone() + "/" + value),
                        store.clone(),
                        Some(passphrase),
                        )
                        .expect("Failed to rename file");
                    Ok(())
                }
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
                    //for now, we are using file path instead of account id to update the entry.
                    //TODO we need to make use account id instead of file path in the future.
                    // let updated_entry=get_entry(&*store.lock()?.lock()?, &file_path_for_temp_id).unwrap();
                    let password=updated_entry.secret(&*store.lock()?.lock()?, Some(passphrase.clone())).unwrap();
                    let mut data=serde_json::to_value(updated_entry).unwrap();
                    data.as_object_mut().unwrap().insert("password".to_string(),serde_json::Value::String(password));
                    let edit_response = EditResponse {
                        acknowledgement: request.acknowledgement,
                        data,
                        status: Status::Success,
                        resource: Resource::Account,
                        id:request.id,
                       meta:None,
                    };
                    let json = serde_json::to_string(&edit_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    Ok(())
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
        let username = request.username;
        let passphrase = request.passphrase;
        let store={
                let stores_locked=stores.lock()?;
                let filtered=stores_locked.iter().filter(|s|s.lock().unwrap().get_name()==&username).collect::<Vec<_>>();
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
        let verified = check_passphrase(&store.clone(), Some(username), &passphrase);
        let acknowledgement = request.acknowledgement;
        let status = {
            if let Ok(verified) = verified {
                if verified {
                    Status::Success
                } else {
                    Status::Failure
                }
            } else {
                Status::Error
            }
        };
        let json = serde_json::json!({"status": status,"acknowledgement": acknowledgement.unwrap_or("".to_string()),});
        let json = serde_json::to_string(&json).unwrap();
        let encoded = encode_message(&json.to_string());
        thread::sleep(time::Duration::from_millis(200));
        send_message(&encoded);
        return Ok(store);
}
fn handle_logout_request(request:LogoutRequest,store: &PasswordStoreType)->pass::Result<()>{
    info!("logout request received for store: {:?}",store.lock()?.lock()?.get_name());
    let acknowledgement = request.acknowledgement;
    let status = Status::Success;
    let json = serde_json::json!({"status": status,"acknowledgement": acknowledgement.unwrap_or("".to_string()),});
    let json = serde_json::to_string(&json).unwrap();
    let encoded = encode_message(&json.to_string());
    send_message(&encoded);
    Ok(())
}
fn handle_create_request(request: CreateRequest, store: &PasswordStoreType) -> pass::Result<()> {
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
                    let (status,data) = match create_password_entry_with_passphrase(
                        value.as_str().map(|s| s.to_owned()),
                        Some(domain.clone() + "/" + &username),
                        store.clone(),
                        None,
                        Some(passphrase.clone()),
                        ){
                        Ok(password_entry)=>{
                            status = Status::Success;
                            let mut entry_data = serde_json::to_value(&password_entry).unwrap();
                            entry_data.as_object_mut().unwrap().insert(
                                "password".to_owned(),
                                serde_json::Value::String(
                                    password_entry
                                    .secret(&*store.lock()?.lock()?, Some(passphrase.clone()))
                                    .unwrap_or("failed to decrypt password".to_string()),
                                    ),
                                    );
                            data = entry_data;
                            (status,data)
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
                    let json = serde_json::to_string(&create_response).unwrap();
                    let encoded = encode_message(&json.to_string());
                    send_message(&encoded);
                    return Ok(());
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
fn handle_delete_request(request: DeleteRequest, store: &PasswordStoreType) -> pass::Result<()> {
    if let Some(header) = request.header {
        if let Some(passphrase) = header.get("passphrase").cloned() {
            let id = request.id;
            let acknowledgement = request.acknowledgement;
            let (status, data) = {
                if let Ok(entry_data) =
                    delete_password_entry(store.clone(), &(id), Some(passphrase))
                    {
                        (Status::Success, Some(entry_data))
                    } else {
                        (Status::Failure, None)
                    }
            };
            let delete_response = DeleteResponse {
                acknowledgement,
                data: data
                    .map(|data| serde_json::to_value(data).unwrap())
                    .unwrap_or_default(),
                    status,
            };
            let json = serde_json::to_string(&delete_response).unwrap();
            let encoded = encode_message(&json.to_string());
            send_message(&encoded);
            Ok(())
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
        let request_result={if let Ok(request) = serde_json::from_value::<RequestEnum>(received_message) {
            info!("request received: {:?}", request);
            if let Some(store)=store_opt.as_ref(){
                match request.clone() {
                    RequestEnum::Get(request) => handle_get_request(request, store),
                    RequestEnum::Search(request) => handle_search_request(request, store),
                    RequestEnum::Fetch(request) => handle_fetch_request(request, store),
                    RequestEnum::Init(request) => {
                        let init_res=handle_init_request(request);
                        if init_res.is_ok(){
                            stores=init_res.unwrap();
                            Ok(())
                        }else{
                            Err(init_res.unwrap_err())
                        }
                    },
                    RequestEnum::Login(request) => {
                        let store_res=handle_login_request(request,&stores);
                        if store_res.is_ok(){
                            store_opt=Some(store_res.unwrap());
                            Ok(())
                        }else{
                            Err(store_res.unwrap_err())
                        }
                    },
                    RequestEnum::Logout(request) => {
                        let res=handle_logout_request(request,store);
                        if res.is_ok(){
                            store_opt=None;
                        Ok(())
                        }else{
                            Err(res.unwrap_err())
                        }
                    },
                    RequestEnum::Create(request) => handle_create_request(request, store),
                    RequestEnum::Delete(request) => handle_delete_request(request, store),
                    RequestEnum::Edit(request) => handle_edit_request(request, store),
                    _ => {
                        Err(pass::Error::Generic("Unknown request"))
                    }
                }
            }else {
                match request.clone() {
                    RequestEnum::Init(request) => {
                        let init_res=handle_init_request(request);
                        if init_res.is_ok(){
                            stores=init_res.unwrap();
                            Ok(())
                        }else{
                            Err(init_res.unwrap_err())
                        }
                    },
                    RequestEnum::Login(request) => {
                        let store_res=handle_login_request(request,&stores);
                        if store_res.is_ok(){
                            store_opt=Some(store_res.unwrap());
                            Ok(())
                        }else{
                            Err(store_res.unwrap_err())
                        }
                    },
                    _ => {
                        error!("Only login request could be accepted when no store has been set. Request was: {:?}", request);
                        Err(pass::Error::Generic("Only login request could be accepted when no store has been set"))
                    }
                }
            }
        } else {
            Err(pass::Error::Generic("Failed to parse message"))
        }
        };
        if request_result.is_err(){
            error!("Error: {:?}", request_result.unwrap_err());
            continue;
        }else{
            info!("Request processed with result: {:?}", request_result.unwrap());
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
    // TODO 
    // following is the temporary solution as id is not used yet. file name will become id of an
    // entry so should never change in future implementation
    // username, password, domain or other fields should exist as some key value pairs in each file named
    // with unique id
    let path=Path::new(id);
    let mut id=id.to_string();
    let parent=path.parent().unwrap().file_name().unwrap().to_str().unwrap();
    let old_name=parent.to_string()+"/"+path.file_stem().unwrap().to_str().unwrap();
    let entry=get_entry(&*store.lock().unwrap().lock().unwrap(), &old_name).unwrap();
    let new_name={
    if let Some(new_name)=new_name.as_ref(){
        if let Some(domain)=domain.as_ref(){
            let name=domain.to_owned()+"/"+new_name;
            if name != old_name{
                Some(name)
            }else{
                None
            }
        }else{
            let name=path.parent().unwrap().file_stem().unwrap().to_str().unwrap().to_string()+"/"+new_name;
            if name != old_name{
                Some(name)
            }else{
                None
            }
        }
    }else if domain.is_some(){
        let name=domain.unwrap()+"/"+path.file_stem().unwrap().to_str().unwrap();
        if name != old_name{
            Some(name)
        }else{
            None
        }
    }else{
        None
    }
    };
    if new_name.is_some(){
        do_rename_file(&old_name, &new_name.clone().unwrap(), store.clone(), passphrase.clone())?;
        info!("renamed file from {:?} to {:?}",old_name,new_name.clone().unwrap());
        id=new_name.unwrap();
    }else{
        //TODO we need to make use account id instead of file path in the future.
        id=old_name;
    }
    let res=if let Some(password) = password {
       change_password(
            &password,
            &id,
            store.clone(),
            passphrase.clone(),
        )
    } 
    else {
        Ok(())
    };
    res.map(|_| get_entry(&*store.lock().unwrap().lock().unwrap(), &id).unwrap())
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

    if let Some(note) = note {
        password = format!("{password}\n{note}");
    }
    if password.contains("otpauth://") {
        error!("It seems like you are trying to save a TOTP code to the password store. This will reduce your 2FA solution to just 1FA, do you want to proceed?");
    }
    _new_password_save(path.as_ref(), password.as_ref(), store)
}
fn create_password_entry_with_passphrase(
    password: Option<String>,
    path: Option<String>,
    store: PasswordStoreType,
    note: Option<String>,
    passphrase: Option<String>,
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

    if let Some(note) = note {
        password = format!("{password}\n{note}");
    }
    if password.contains("otpauth://") {
        error!("It seems like you are trying to save a TOTP code to the password store. This will reduce your 2FA solution to just 1FA, do you want to proceed?");
    }
    new_password_save_with_passphrase(path.as_ref(), password.as_ref(), store, passphrase)
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
fn new_password_save_with_passphrase(
    path: &str,
    password: &str,
    store: PasswordStoreType,
    passphrase: Option<String>,
    ) -> pass::Result<PasswordEntry> {
    let entry = store.lock()?.lock()?.new_password_file_with_passphrase(
        path.as_ref(),
        password.as_ref(),
        passphrase,
        );
    entry
}

fn change_password(
    password: &str,
    entry_filename: &str,
    store: PasswordStoreType,
    passphrase: Option<String>,
    ) -> pass::Result<()> {
    let password_entry_opt = get_entry(&*store.lock()?.lock()?, entry_filename);
    if password_entry_opt.is_none() {
        return Err("No password entry found".into());
    }
    let password_entry = password_entry_opt.unwrap();
    let r = password_entry.update_with_passphrase(
        password.to_string(),
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
    let matching = passwords.iter().filter(|p| matches(&p.name, query));
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
    let matching = passwords.iter().find(|p| matches(&p.name, path)).cloned();
    return matching;
}
pub fn remove_entry(store: &mut PasswordStore, path: &str) -> Option<PasswordEntry> {
    let passwords = &mut store.passwords;
    fn normalized(s: &str) -> String {
        s.to_lowercase()
    }
    fn matches(s: &str, p: &str) -> bool {
        normalized(s).as_str() == normalized(p).as_str()
    }
    if let Some(idx) = passwords
        .iter()
            .position(|p| matches(p.path.to_str().unwrap(), path))
            {
                let matching = passwords.remove(idx);
                return Some(matching);
            } else {
                return None;
            }
}
fn delete_password_entry(
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
