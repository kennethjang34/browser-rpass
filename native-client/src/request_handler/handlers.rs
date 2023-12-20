use rpass::{crypto::Handler, pass::CUSTOM_FIELD_PREFIX};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub use super::util::*;

use browser_rpass::{request::*, response::*};
use log::*;
use rpass::pass::{self, Error, PasswordEntry, PasswordStore};
use serde_json::json;

use crate::{store_api::*, util::*, PasswordStoreType, StoreListType};

pub fn handle_edit_request(
    request: EditRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<EditResponse> {
    let value = &request.value;
    let resource = &request.resource;
    match resource {
        Resource::Account => {
            let domain = value
                .get(&DataFieldType::Domain)
                .map(|v| v.as_str())
                .flatten();
            let username = value
                .get(&DataFieldType::Username)
                .map(|v| v.as_str())
                .flatten();
            let password = value
                .get(&DataFieldType::Password)
                .map(|v| v.as_str())
                .flatten();
            let note = value
                .get(&DataFieldType::Note)
                .map(|v| v.as_str())
                .flatten();
            let custom_fields = value
                .get(&DataFieldType::CustomField)
                .map(|v| v.as_object())
                .flatten();
            let updated_data = store.lock()?.lock()?.update_default_entry_fields(
                &request.id,
                domain,
                username,
                password,
                note,
                custom_fields,
                passphrase_provider,
            );

            let mut data = HashMap::new();
            match updated_data {
                Ok(updated_data) => {
                    data.insert(DataFieldType::UpdatedFields, updated_data.clone());
                    let edit_response = EditResponse {
                        acknowledgement: request.acknowledgement,
                        data,
                        status: Status::Success,
                        resource: Resource::Account,
                        id: request.id,
                        meta: None,
                    };
                    Ok(edit_response)
                }
                Err(err) => {
                    error!("Failed to update entry: {:?}", err);
                    Err(err)
                }
            }
        }
        _ => {
            return Err(pass::Error::from(
                "Currently only resource type of Account is supported",
            ));
        }
    }
}

pub fn handle_get_request(
    request: GetRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<GetResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    let id = request.id;
    match resource {
        Resource::Account => {
            let locked_store = store.lock()?;
            let locked_store = locked_store.lock()?;
            let encrypted_password_entry = locked_store.get_entry(&id);
            let password_entry =
                encrypted_password_entry.and_then(|encrypted_password_entry: PasswordEntry| {
                    let json_value_res: serde_json::Result<
                        serde_json::Map<String, serde_json::Value>,
                    > = (&encrypted_password_entry).try_into();
                    if let Ok(mut json_value) = json_value_res {
                        json_value.insert(
                            "password".to_owned(),
                            serde_json::Value::String(
                                encrypted_password_entry
                                    .secret(&locked_store, passphrase_provider)
                                    .unwrap_or("failed to decrypt password".to_string()),
                            ),
                        );
                        Ok(json_value)
                    } else {
                        Err(pass::Error::from(
                            "failed to convert password entry to serde_json::Value",
                        ))
                    }
                });
            let mut data = HashMap::new();
            let get_response = {
                if let Ok(data_value) = password_entry.map(|data| data.into()) {
                    data.insert(DataFieldType::Data, data_value);
                    GetResponse {
                        data,
                        meta: Some(json!({"id":id})),
                        resource,
                        acknowledgement,
                        status: Status::Success,
                    }
                } else {
                    GetResponse {
                        data,
                        meta: Some(json!({"id":id})),
                        resource,
                        acknowledgement,
                        status: Status::Failure,
                    }
                }
            };
            return Ok(get_response);
        }
        _ => {
            return Err(pass::Error::from(
                "Currently only resource type of Account is supported",
            ));
        }
    };
}
pub fn handle_search_request(
    request: SearchRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<SearchResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    let query = request.query.unwrap_or("".to_string());
    match resource {
        Resource::Account => {
            let encrypted_password_entries = &filter_entries(&store, &query)?;
            let locked_store = store.lock()?;
            let locked_store = &*locked_store.lock()?;
            let decrypted_password_entries = encrypted_password_entries
                .iter()
                .filter_map(|encrypted_password_entry| {
                    let json_value_res: serde_json::Result<
                        serde_json::Map<String, serde_json::Value>,
                    > = encrypted_password_entry.try_into();
                    if let Ok(mut mut_obj) = json_value_res {
                        mut_obj.insert(
                            "password".to_owned(),
                            serde_json::Value::String(
                                encrypted_password_entry
                                    .secret(locked_store, passphrase_provider.clone())
                                    .unwrap_or("failed to decrypt password".to_string()),
                            ),
                        );
                        Some(mut_obj.into())
                    } else {
                        None
                    }
                })
                .collect::<Vec<serde_json::Value>>();
            let search_response = {
                let mut data = HashMap::new();
                if let Ok(data_value) = serde_json::to_value(decrypted_password_entries.clone()) {
                    if let Some(data_arr) = data_value.as_array().cloned() {
                        data.insert(DataFieldType::Data, data_arr.into());
                        SearchResponse {
                            data,
                            acknowledgement: acknowledgement.clone(),
                            status: Status::Success,
                            resource,
                            meta: Some(json!({"query":query.clone()})),
                        }
                    } else {
                        SearchResponse {
                            data,
                            acknowledgement: acknowledgement.clone(),
                            status: Status::Failure,
                            resource,
                            meta: Some(
                                json!({"query":query.clone(), "error":format!("failed to parse data as array. Data: {:?}", data_value)}),
                            ),
                        }
                    }
                } else {
                    SearchResponse {
                        data,
                        acknowledgement,
                        status: Status::Failure,
                        resource,
                        meta: Some(
                            json!({"query":query, "error":format!("failed to convert data to serde_json::Value. Data: {:?}", decrypted_password_entries)}),
                        ),
                    }
                }
            };
            return Ok(search_response);
        }
        _ => {
            return Err(pass::Error::from(
                "Currently only resource type of Account is supported",
            ));
        }
    };
}
pub fn handle_fetch_request(
    request: FetchRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<FetchResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    match resource {
        Resource::Account => {
            let locked_store = store.lock()?;
            let locked_store = &*locked_store.lock()?;
            let encrypted_password_entries = locked_store.get_entries(None)?;
            let decrypted_password_entries = encrypted_password_entries
                        .iter()
                        .filter_map(|encrypted_password_entry| {
                            let json_value_res: serde_json::Result<
                                serde_json::Value,
                            > = encrypted_password_entry.try_into();
                            if let Ok(mut json_value) =
                                json_value_res
                            {
                                if let Ok(decrypted) = encrypted_password_entry
                                    .secret(locked_store, passphrase_provider.clone())
                                {
                                    if let Ok(decrypted) =
                                        serde_json::from_str::<serde_json::Value>(&decrypted)
                                    {
                                        merge_json(&mut json_value, &decrypted);
                                        Some(json_value)
                                    } else {
                                        error!("failed to parse decrypted json string into serde::Value. Json String: {:?}", decrypted);
                                        return None;
                                    }
                                }else{
                                    error!("failed to decrypt password entry: {:?}", encrypted_password_entry);
                                    return None;
                                }
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<serde_json::Value>>();
            let fetch_response = {
                let mut data = HashMap::new();
                if let Ok(data_value) = serde_json::to_value(decrypted_password_entries) {
                    if let Some(data_arr) = data_value.as_array().cloned() {
                        data.insert(DataFieldType::Data, json!(data_arr));
                        FetchResponse {
                            data,
                            meta: Some(json!({"custom_field_prefix":CUSTOM_FIELD_PREFIX})),
                            resource,
                            acknowledgement,
                            status: Status::Success,
                        }
                    } else {
                        FetchResponse {
                            data,
                            meta: None,
                            resource,
                            acknowledgement,
                            status: Status::Failure,
                        }
                    }
                } else {
                    FetchResponse {
                        data,
                        meta: None,
                        resource,
                        acknowledgement,
                        status: Status::Failure,
                    }
                }
            };
            return Ok(fetch_response);
        }
        _ => {
            error!("requsted resource: {:?} not supported", resource);
            return Err(pass::Error::from(
                "Currently only resource type of Account is supported",
            ));
        }
    };
}
pub fn handle_init_request(request: InitRequest) -> pass::Result<StoreListType> {
    let home_dir = request.config.get("home_dir");
    let home = {
        if home_dir.is_some() {
            home_dir.map(|s| PathBuf::from(s))
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
            store_dir
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
            password_store_signing_key
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
    let (config, config_file_location) = config_res?;

    let stores = PasswordStore::get_stores(&config, &home);
    if let Err(err) = stores {
        error!("{:?}", err);
        return Err(err);
    }

    let stores: StoreListType = Arc::new(Mutex::new(
        stores?
            .into_iter()
            .map(|s| Arc::new(Mutex::new(s)))
            .collect(),
    ));

    if !config_file_location.exists() && stores.lock()?.len() == 1 {
        let mut config_file_dir = config_file_location.clone();
        config_file_dir.pop();
        if let Err(err) = std::fs::create_dir_all(config_file_dir) {
            error!("{:?}", err);
            return Err(pass::Error::from(err));
        }
        if let Err(err) = pass::save_config(stores.clone(), &config_file_location) {
            error!("{:?}", err);
            return Err(err);
        }
    }
    Ok(stores)
}
pub fn handle_login_request(
    request: LoginRequest,
    stores: &StoreListType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<PasswordStoreType> {
    let store_name = request.store_id;
    let store = {
        let stores_locked = stores.lock()?;
        let filtered = stores_locked
            .iter()
            .filter(|s| {
                s.lock()
                    .map(|s| s.get_name() == &store_name)
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        if filtered.len() == 0 {
            return Err(Error::GenericDyn(format!(
                "No store found for username: {}",
                store_name
            )));
        }
        filtered[0].clone()
    };
    let store: PasswordStoreType = Arc::new(Mutex::new(store));
    let res = store.lock()?.lock()?.reload_password_list();
    if let Err(err) = res {
        return Err(err);
    }

    // verify that the git config is correct (note: name field is not used for gpg
    // signing/encryption per se, but it is to record the user who made the commit)
    if !store.lock()?.lock()?.has_configured_username() {
        Err(Error::GenericDyn(
            "Git user.name and user.email must be configured".to_string(),
        ))?;
    }
    for password in &store.lock()?.lock()?.passwords {
        if password.is_in_git == pass::RepositoryStatus::NotInRepo {
            Err(Error::GenericDyn(
                format!(
                    "Password entry: {:?}  not found in the current store",
                    password
                )
                .to_string(),
            ))?;
        }
    }
    let verified = store.lock()?.lock()?.try_passphrase(passphrase_provider);
    match verified {
        Ok(verified) => {
            if verified {
                return Ok(store);
            } else {
                return Err(Error::GenericDyn("Failed to verify passphrase".to_string()));
            }
        }
        Err(e) => {
            debug!("Failed to verify passphrase: {:?}", e);
            return Err(Error::GenericDyn(format!(
                "Failed to verify passphrase: {:?}",
                e
            )));
        }
    }
}
pub fn handle_logout_request(
    request: LogoutRequest,
    _store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<()> {
    let _acknowledgement = request.acknowledgement;
    let _status = Status::Success;
    if let Some(passphrase_provider) = passphrase_provider {
        passphrase_provider.passphrases.write().unwrap().clear();
    }
    Ok(())
}
pub fn handle_create_request(
    request: CreateRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<CreateResponse> {
    let username = request.username;
    let domain = request.domain;
    let note = request.note;
    let password = request.password;
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    let custom_fields = request.custom_fields;
    let status;
    match resource {
        Resource::Account => {
            let locked_store = store.lock()?;
            let mut locked_store = locked_store.lock()?;
            let mut data = HashMap::new();
            let status = match locked_store.create_entry(
                username.as_deref(),
                password.as_deref(),
                domain.as_deref(),
                note.as_deref(),
                custom_fields,
                passphrase_provider.clone(),
            ) {
                Ok(entry) => {
                    if let Ok(mut entry_data) = serde_json::from_str(
                        entry
                            .secret(&locked_store, passphrase_provider.clone())?
                            .as_str(),
                    ) {
                        let entry_meta_res: serde_json::Result<serde_json::Value> =
                            (&entry).try_into();
                        if let Ok(entry_meta) = entry_meta_res.as_ref() {
                            merge_json(&mut entry_data, entry_meta);
                            status = Status::Success;
                            data.insert(DataFieldType::Data, entry_data);
                            status
                        } else {
                            error!(
                                "failed to convert entry to serde_json::Value. response: {:?}",
                                entry_meta_res
                            );
                            status = Status::Failure;
                            status
                        }
                    } else {
                        status = Status::Failure;
                        status
                    }
                }
                Err(err) => {
                    status = Status::Failure;
                    error!("Failed to create password entry: {:?}", err);
                    status
                }
            };
            let create_response: CreateResponse = CreateResponse {
                acknowledgement,
                data,
                meta: None,
                resource: Resource::Account,
                status,
            };
            return Ok(create_response);
        }
        _ => {
            return Err(pass::Error::from(
                "Currently only resource type of Account is supported",
            ));
        }
    };
}
pub fn handle_delete_request(
    request: DeleteRequest,
    store: &PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<DeleteResponse> {
    let id = request.id;
    let acknowledgement = request.acknowledgement;
    let mut data = HashMap::new();
    let status = {
        let res = store
            .lock()?
            .lock()?
            .delete_entry(&(id), passphrase_provider);
        match res {
            Ok(entry_data) => {
                data.insert(
                    DataFieldType::ResourceID,
                    serde_json::to_value(entry_data.id.clone()).unwrap_or_default(),
                );
                data.insert(DataFieldType::Data, serde_json::to_value(entry_data)?);
                Status::Success
            }
            Err(e) => {
                error!("Failed to delete entry: {:?}", e);
                Status::Failure
            }
        }
    };
    let delete_response = DeleteResponse {
        acknowledgement,
        data,
        status,
    };
    Ok(delete_response)
}
