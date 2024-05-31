use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub use super::util::*;

use browser_rpass::{request::*, response::*};
use log::*;
use rpass::{
    crypto::Handler,
    pass::{self, PasswordStore},
};
use serde_json::json;

use crate::{request_handler::*, util::ToJson, StoreListType};
fn get_store(request: &RequestEnum, stores: &StoreListType) -> Option<Arc<Mutex<PasswordStore>>> {
    let store_id = request.get_store_id();
    if store_id.is_none() {
        return None;
    }
    let store_id = store_id.unwrap();
    let store = stores.lock().unwrap();
    let store = store
        .iter()
        .find(|store| store.lock().unwrap().get_name() == &store_id);
    if let Some(store) = store {
        Some(store.clone())
    } else {
        None
    }
}
pub fn listen_to_native_messaging(
    stores: StoreListType,
    passphrase_provider: Option<Handler>,
    home: Option<PathBuf>,
    config_file_location: PathBuf,
) -> pass::Result<()> {
    loop {
        let received_message_res = get_message();
        if received_message_res.is_err() {
            continue;
        }
        let received_message = received_message_res?;
        let deserde = serde_json::from_value::<RequestEnum>(received_message.clone());
        if let Ok(request) = deserde {
            let request_result = {
                let target_store = get_store(&request, &stores);
                match request.clone() {
                    RequestEnum::Init(request) => {
                        let response = handle_init_request(request.clone());
                        let mut detail = HashMap::new();
                        if response.is_ok() {
                            let keys = response?;
                            let mut store_ids = Vec::new();
                            {
                                for store in stores.clone().lock().unwrap().iter() {
                                    store_ids.push(store.lock().unwrap().get_name().clone());
                                }
                            }
                            detail.insert(
                                DataFieldType::StoreIDList,
                                serde_json::to_value(store_ids)?,
                            );
                            let keys = keys
                                .into_iter()
                                .filter_map(|k| {
                                    if k.is_not_usable() {
                                        return None;
                                    } else {
                                        Some(k.to_json())
                                    }
                                })
                                .collect::<Vec<_>>();
                            detail.insert(DataFieldType::Keys, serde_json::to_value(keys)?);
                            let response = ResponseEnum::InitResponse(InitResponse {
                                status: Status::Success,
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                            });
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            detail.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::InitResponse(InitResponse {
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::CreateStore(request) => {
                        let response = handle_create_store_request(
                            request.clone(),
                            passphrase_provider.clone(),
                            &stores,
                            &home,
                            &config_file_location,
                        );
                        let mut detail = HashMap::new();
                        if response.is_ok() {
                            let response = ResponseEnum::CreateStoreResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            error!("Failed to create store: {:?}", response);
                            detail.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::CreateStoreResponse(CreateStoreResponse {
                                status: Status::Failure,
                                store_path: request
                                    .get_store_path()
                                    .map(|s| PathBuf::from(s))
                                    .unwrap_or(PathBuf::new()),
                                store_id: request.get_store_name(),
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::DeleteStore(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_delete_store_request(
                            request.clone(),
                            passphrase_provider.clone(),
                            &stores,
                            &home,
                            &config_file_location,
                            &store,
                        );
                        let mut detail = HashMap::new();
                        if response.is_ok() {
                            let response = ResponseEnum::DeleteStoreResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            error!("Failed to create store: {:?}", response);
                            detail.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::DeleteStoreResponse(DeleteStoreResponse {
                                status: Status::Failure,
                                store_id: request.store_id.clone(),
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Get(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_get_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        if response.is_ok() {
                            let response = ResponseEnum::GetResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            let mut detail = HashMap::new();
                            detail
                                .insert(DataFieldType::ErrorMessage, json!(response.unwrap_err()));
                            detail.insert(DataFieldType::Request, json!(request));
                            let response = ResponseEnum::GetResponse(GetResponse {
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                                resource: Resource::Password,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Search(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_search_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        if response.is_ok() {
                            let response = response?;
                            let response = ResponseEnum::SearchResponse(response);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            let mut data = HashMap::new();
                            data.insert(DataFieldType::Data, json!([]));
                            let response = ResponseEnum::SearchResponse(SearchResponse {
                                store_id: request.store_id.clone().unwrap(),
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail: data,
                                resource: request.resource,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Fetch(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_fetch_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                            &stores,
                        );
                        if response.is_ok() {
                            let response = response?;
                            let response = ResponseEnum::FetchResponse(response);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            let mut data = HashMap::new();
                            data.insert(DataFieldType::Data, json!([]));
                            let response = ResponseEnum::FetchResponse(FetchResponse {
                                store_id: request.store_id.clone().unwrap(),
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail: data,
                                resource: request.resource,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Login(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let store_res = handle_login_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        let mut data = HashMap::new();
                        if store_res.is_ok() {
                            let response = ResponseEnum::LoginResponse(LoginResponse {
                                status: Status::Success,
                                acknowledgement: request.acknowledgement.clone(),
                                store_id: request.store_id.clone().unwrap(),
                                detail: data,
                            });
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            data.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(store_res.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::LoginResponse(LoginResponse {
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                store_id: request.store_id.clone().unwrap(),
                                detail: data,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Logout(request) => {
                        let res = handle_logout_request(
                            request.clone(),
                            &target_store,
                            passphrase_provider.clone(),
                        );
                        let mut data = HashMap::new();
                        if res.is_ok() {
                            let response = ResponseEnum::LogoutResponse(LogoutResponse {
                                status: Status::Success,
                                acknowledgement: request.acknowledgement.clone(),
                                store_id: request.store_id,
                                detail: data,
                            });
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            data.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(res.unwrap_err()).unwrap(),
                            );
                            let response = LogoutResponse {
                                store_id: request.store_id,
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail: data,
                            };
                            send_as_json(&response)?;
                            Err(ResponseEnum::LogoutResponse(response))
                        }
                    }
                    RequestEnum::Create(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_create_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        let mut data = HashMap::new();
                        if response.is_ok() {
                            let response = ResponseEnum::CreateResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            data.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::CreateResponse(CreateResponse {
                                status: Status::Failure,
                                store_id: request.store_id.clone().unwrap(),
                                acknowledgement: request.acknowledgement.clone(),
                                detail: data,
                                resource: request.resource,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Delete(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_delete_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        let mut data = HashMap::new();
                        if response.is_ok() {
                            let response = ResponseEnum::DeleteResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            data.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::DeleteResponse(DeleteResponse {
                                status: Status::Failure,
                                deleted_resource_id: request.instance_id,
                                acknowledgement: request.acknowledgement.clone(),
                                detail: data,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    RequestEnum::Edit(request) if target_store.is_some() => {
                        let store = target_store.unwrap();
                        let response = handle_edit_request(
                            request.clone(),
                            &store,
                            passphrase_provider.clone(),
                        );
                        let mut detail = HashMap::new();
                        if response.is_ok() {
                            let response = ResponseEnum::EditResponse(response?);
                            send_as_json(&response)?;
                            Ok(response)
                        } else {
                            detail.insert(
                                DataFieldType::ErrorMessage,
                                serde_json::to_value(response.unwrap_err()).unwrap(),
                            );
                            let response = ResponseEnum::EditResponse(EditResponse {
                                store_id: request.store_id.unwrap(),
                                instance_id: request.instance_id,
                                status: Status::Failure,
                                acknowledgement: request.acknowledgement.clone(),
                                detail,
                                resource: request.resource,
                            });
                            send_as_json(&response)?;
                            Err(response)
                        }
                    }
                    _ => {
                        let mut data = HashMap::new();
                        data.insert(DataFieldType::ErrorMessage, json!("Unknown request"));
                        data.insert(DataFieldType::Request, json!(request));
                        error!("Unknown Request: {:?}", request);
                        Err(ResponseEnum::GenericError(GenericError {
                            status: Status::Failure,
                            acknowledgement: request.get_acknowledgement(),
                            detail: data,
                        }))
                    }
                }
            };
            if let Err(error) = request_result {
                send_as_json(&error)?;
            }
        } else {
            error!("Failed to parse message: {:?}", received_message);
        }
    }
}
