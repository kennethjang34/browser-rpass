use rpass::{
    crypto::{self, Handler, Key},
    git::RepoExt,
    pass::{save_config, Recipient, CUSTOM_FIELD_PREFIX},
};
use std::{
    collections::HashMap,
    fs::{remove_dir, remove_dir_all},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub use super::util::*;

use browser_rpass::{request::*, response::*};
use log::*;
use rpass::pass::{self, Error, PasswordEntry, PasswordStore};
use serde_json::json;

use crate::{store_api::*, util::*, StoreListType};

pub fn handle_edit_request(
    request: EditRequest,
    store: &Arc<Mutex<PasswordStore>>,
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
            let updated_data = store.lock()?.update_default_entry_fields(
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
                        store_id: store.lock()?.get_name().clone(),
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
    store: &Arc<Mutex<PasswordStore>>,
    passphrase_provider: Option<Handler>,
) -> pass::Result<GetResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    let id = request.id;
    match resource {
        Resource::Account => {
            let locked_store = store.lock()?;
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
    store: &Arc<Mutex<PasswordStore>>,
    passphrase_provider: Option<Handler>,
) -> pass::Result<SearchResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    let query = request.query.unwrap_or("".to_string());
    match resource {
        Resource::Account => {
            let encrypted_password_entries = &filter_entries(&store, &query)?;
            let locked_store = store.lock()?;
            // let locked_store = &*locked_store.lock()?;
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
                                    .secret(&locked_store, passphrase_provider.clone())
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
                            store_id: locked_store.get_name().clone(),
                            data,
                            acknowledgement: acknowledgement.clone(),
                            status: Status::Success,
                            resource,
                            meta: Some(json!({"query":query.clone()})),
                        }
                    } else {
                        SearchResponse {
                            store_id: locked_store.get_name().clone(),
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
                        store_id: locked_store.get_name().clone(),
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
    store: &Arc<Mutex<PasswordStore>>,
    passphrase_provider: Option<Handler>,
    store_list: &StoreListType,
) -> pass::Result<FetchResponse> {
    let resource = request.resource;
    let acknowledgement = request.acknowledgement;
    match resource {
        Resource::Account => {
            let store_path = {
                let mut locked_store = store.lock()?;
                locked_store.reload_password_list()?;
                locked_store.get_store_path()
            };
            let mut substores = vec![];
            for candidate_store in store_list.lock()?.iter() {
                let locked_candidate_store = candidate_store.lock()?;
                if locked_candidate_store
                    .get_store_path()
                    .parent()
                    .is_some_and(|p| p == store_path)
                {
                    substores.push(locked_candidate_store.get_name().clone());
                }
            }

            let mut locked_store = store.lock()?;
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
                                if let Some(passphrase_provider)=passphrase_provider.clone(){
                                    if passphrase_provider.passphrases.read().unwrap().get(locked_store.get_name()).is_none(){
                                        locked_store.try_passphrase(Some(passphrase_provider.clone())).unwrap();
                                    }
                                }
                                if let Ok(decrypted) = encrypted_password_entry
                                    .secret(&locked_store, passphrase_provider.clone())
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
                if let Ok(entries) = serde_json::to_value(decrypted_password_entries) {
                    if let Some(entry_arr) = entries.as_array().cloned() {
                        data.insert(DataFieldType::Data, json!(entry_arr));
                        data.insert(DataFieldType::SubStore, json!(substores));
                        FetchResponse {
                            store_id: locked_store.get_name().clone(),
                            data,
                            meta: Some(json!({"custom_field_prefix":CUSTOM_FIELD_PREFIX})),
                            resource,
                            acknowledgement,
                            status: Status::Success,
                        }
                    } else {
                        FetchResponse {
                            store_id: locked_store.get_name().clone(),
                            data,
                            meta: None,
                            resource,
                            acknowledgement,
                            status: Status::Failure,
                        }
                    }
                } else {
                    FetchResponse {
                        store_id: locked_store.get_name().clone(),
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
#[allow(unused_variables)]
pub fn handle_init_request(request: InitRequest) -> pass::Result<Vec<Box<dyn Key>>> {
    let keys = crypto::get_keys(crypto::CryptoImpl::GpgMe)?;
    Ok(keys)
}
pub fn handle_login_request(
    _request: LoginRequest,
    store: &Arc<Mutex<PasswordStore>>,
    passphrase_provider: Option<Handler>,
) -> pass::Result<&Arc<Mutex<PasswordStore>>> {
    let res = store.lock()?.reload_password_list();
    if let Err(err) = res {
        return Err(err);
    }

    // verify that the git config is correct (note: name field is not used for gpg
    // signing/encryption per se, but it is to record the user who made the commit)
    if !store.lock()?.has_configured_username() {
        error!("Git user.name and user.email must be configured");
        Err(Error::GenericDyn(
            "Git user.name and user.email must be configured".to_string(),
        ))?;
    }
    let verified = store.lock()?.try_passphrase(passphrase_provider);
    match verified {
        Ok(verified) => {
            if verified {
                return Ok(store);
            } else {
                return Err(Error::GenericDyn("Failed to verify passphrase".to_string()));
            }
        }
        Err(e) => {
            return Err(Error::GenericDyn(format!(
                "Failed to verify passphrase: {:?}",
                e
            )));
        }
    }
}
pub fn handle_logout_request(
    request: LogoutRequest,
    store: &Option<Arc<Mutex<PasswordStore>>>,
    passphrase_provider: Option<Handler>,
) -> pass::Result<()> {
    let _acknowledgement = request.acknowledgement.clone();
    let _status = Status::Success;
    if let Some(mut passphrase_provider) = passphrase_provider.clone() {
        if let Some(_store_id) = request.store_id {
            if store.is_none() {
                return Err(pass::Error::NoneError);
            }
            let store = store.as_ref().unwrap();
            if let Some(login_recipient) = store.lock()?.get_login_recipient() {
                let login_key_id = &login_recipient.key_id;
                passphrase_provider.remove_passphrase(login_key_id, true)?;
            }
            if let Ok(repo) = store.lock()?.repo() {
                let from_signing_key = repo.config()?.get_string("user.signingkey").ok();
                if let Some(key) = from_signing_key {
                    passphrase_provider.remove_passphrase(&key, true)?;
                }
                let from_email = repo.config()?.get_string("user.email").ok();
                if let Some(key) = from_email {
                    passphrase_provider.remove_passphrase(&key, true)?;
                }
            }
            // remove all passphrases for valid signing keys
            // TODO: this is quite inefficient, we should only remove the passphrase for the
            // key used to sign the gpg-id file
            for key in store.lock().unwrap().get_valid_gpg_signing_keys() {
                let key = hex::encode(key);
                passphrase_provider.remove_passphrase(&key, true)?;
            }
        } else {
            if let Some(user_id) = request.user_id {
                {
                    let mut ctx = passphrase_provider.create_context().unwrap();
                    let key = ctx.get_secret_key(user_id).unwrap();
                    let subkeys = key.subkeys();
                    for subkey in subkeys {
                        let key_id = subkey.id().unwrap();
                        passphrase_provider
                            .passphrases
                            .write()
                            .unwrap()
                            .remove(key_id);
                    }
                }
            } else {
                let res = passphrase_provider.clear_passphrases();
            }
        }
    }
    Ok(())
}

pub fn handle_create_store_request(
    request: CreateStoreRequest,
    passphrase_provider: Option<Handler>,
    store_list: &StoreListType,
    home: &Option<PathBuf>,
    config_file_location: &Path,
) -> pass::Result<CreateStoreResponse> {
    let crypto = crypto::CryptoImpl::GpgMe.get_crypto_type()?;
    let store_name = request.get_store_name();
    let encryption_key_ids = request.encryption_keys.clone();
    let signer = request.repo_signing_key.as_ref();
    let signer = signer.map(|key_id| {
        Recipient::from(
            &hex::encode(&crypto.get_key(key_id).unwrap().fingerprint().unwrap()),
            &[],
            None,
            &*crypto,
        )
        .unwrap()
    });
    let store_path = {
        if store_list.lock()?.len() == 0 {
            pass::password_dir_raw(&None, home)
        } else {
            pass::password_dir_raw(&None, home).join(&store_name)
        }
    };
    let mut recipients: Vec<Recipient> = vec![];
    let valid_signing_keys = request.valid_signing_keys.clone();
    let mut signing_recipients: Vec<Recipient> = vec![];
    if let Some(valid_signing_keys) = valid_signing_keys {
        for key_id in valid_signing_keys {
            let key_found = crypto.get_key(&key_id)?;
            let fpt = key_found.fingerprint()?;
            let fpt_str = &hex::encode(fpt);
            let recipient = Recipient::from(&fpt_str, &[], None, &*crypto).unwrap();
            signing_recipients.push(recipient);
        }
    }
    for key_id in encryption_key_ids {
        let key_found = crypto.get_key(&key_id)?;
        let fpt = key_found.fingerprint()?;
        let fpt_str = &hex::encode(fpt);
        let recipient = Recipient::from(&fpt_str, &[], None, &*crypto)?;
        recipients.push(recipient);
    }

    let store = PasswordStore::create(
        &request.get_store_name(),
        &Some(store_path.clone()),
        &recipients,
        &signing_recipients,
        &signer,
        home,
        passphrase_provider.clone(),
    )?;
    let current_repo_sig = store.repo()?.signature()?;
    let store_url = store.get_store_path();
    if let Some(parent_store_name) = request.parent_store.as_ref() {
        let parent_store = get_store(parent_store_name, store_list);
        if let Some(parent_store) = parent_store {
            let parent_path = parent_store.lock().unwrap().get_store_path();
            let parent_repo = git2::Repository::open(parent_path)?;
            let submodule = parent_repo.submodule(
                store_url.to_str().unwrap(),
                Path::new(store.get_name()),
                false,
            );
            let mut submodule = submodule.unwrap();
            submodule.clone(None).unwrap();
            submodule.add_finalize().unwrap();
            let mut parents = vec![];
            let parent_commit;
            if let Ok(pc) = parent_repo.find_last_commit() {
                parent_commit = pc;
                parents.push(&parent_commit);
            }
            let mut index = parent_repo.index().unwrap();
            let oid = index.write_tree().unwrap();
            let tree = parent_repo.find_tree(oid)?;
            <git2::Repository as RepoExt>::commit(
                &parent_repo,
                // // don't use parent's signature, use the current repo's signature
                // // this makes it easy to identify which key has been used associated with the newly
                // // created store
                &current_repo_sig,
                "added submodule",
                &tree,
                &parents,
                crypto.as_ref(),
                passphrase_provider.clone(),
            )
            .unwrap();
        }
    }
    let store_ptr = Arc::new(Mutex::new(store));
    {
        store_list.lock()?.push(store_ptr.clone());
    }
    {
        store_ptr.lock()?.reload_password_list()?;
    }

    let mut config_file_dir = config_file_location.to_path_buf();
    let config_save_res = {
        if !config_file_location.exists() && store_list.lock()?.len() == 1 {
            config_file_dir.pop();
            if let Err(err) = std::fs::create_dir_all(config_file_dir) {
                Err(pass::Error::from(err))
            } else {
                if let Err(err) = pass::save_config(store_list.clone(), &config_file_location) {
                    Err(err)
                } else {
                    Ok(())
                }
            }
        } else {
            save_config(store_list.clone(), &config_file_location)
        }
    };
    if config_save_res.is_err() {
        remove_dir_all(store_path.clone())?;
    }

    return Ok(CreateStoreResponse::new(
        request.get_store_name(),
        store_path,
        Status::Success,
        request.get_acknowledgement(),
        None,
        None,
    ));
}
#[allow(unused_variables)]
pub fn handle_delete_store_request(
    request: DeleteStoreRequset,
    passphrase_provider: Option<Handler>,
    store_list: &StoreListType,
    home: &Option<PathBuf>,
    config_file_location: &Path,
    store: &Arc<Mutex<PasswordStore>>,
) -> pass::Result<DeleteStoreResponse> {
    if request.force {
        remove_dir(store.lock()?.get_store_path())?;
    } else {
        remove_dir_all(store.lock()?.get_store_path())?;
    }
    let store_name = store.lock()?.get_name().clone();
    {
        let mut store_list = store_list.lock()?;
        store_list.retain(|s| s.lock().unwrap().get_name() != &store_name);
    }
    save_config(store_list.clone(), &config_file_location)?;
    Ok(DeleteStoreResponse {
        store_id: store_name,
        acknowledgement: request.acknowledgement,
        status: Status::Success,
        data: HashMap::new(),
        meta: None,
    })
}
pub fn handle_create_request(
    request: CreateRequest,
    store: &Arc<Mutex<PasswordStore>>,
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
            let mut locked_store = store.lock()?;
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
                store_id: locked_store.get_name().clone(),
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
    store: &Arc<Mutex<PasswordStore>>,
    passphrase_provider: Option<Handler>,
) -> pass::Result<DeleteResponse> {
    let id = request.id;
    let acknowledgement = request.acknowledgement;
    let mut data = HashMap::new();
    let status = {
        let res = store.lock()?.delete_entry(&(id), passphrase_provider);
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
        deleted_resource_id: id,
        acknowledgement,
        data,
        status,
    };
    Ok(delete_response)
}
