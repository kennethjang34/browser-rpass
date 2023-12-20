use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub use super::util::*;

use browser_rpass::{request::*, response::*};
use log::*;
use rpass::{
    crypto::Handler,
    pass::{self},
};
use serde_json::json;

use crate::{request_handler::*, PasswordStoreType, StoreListType};
pub fn listen_to_native_messaging(
    mut stores: StoreListType,
    passphrases: Option<Arc<RwLock<HashMap<String, String>>>>,
) -> pass::Result<()> {
    trace!("start listening to native messaging");
    let mut store_opt: Option<PasswordStoreType> = None;
    loop {
        let received_message_res = get_message();
        if received_message_res.is_err() {
            continue;
        }
        let received_message = received_message_res?;
        if let Ok(request) = serde_json::from_value::<RequestEnum>(received_message.clone()) {
            let request_result = {
                if let Some(store) = store_opt.as_ref() {
                    let passphrase_provider = Some(Handler::new(
                        passphrases
                            .clone()
                            .unwrap_or(Arc::new(RwLock::new(HashMap::new()))),
                        Some(request.get_type()),
                    ));
                    match request.clone() {
                        RequestEnum::Get(request) => {
                            let response =
                                handle_get_request(request.clone(), store, passphrase_provider);
                            if response.is_ok() {
                                let response = ResponseEnum::GetResponse(response?);
                                send_as_json(&response)?;
                                Ok(response)
                            } else {
                                let mut data = HashMap::new();
                                data.insert(
                                    DataFieldType::ErrorMessage,
                                    json!(response.unwrap_err()),
                                );
                                data.insert(DataFieldType::Request, json!(request));
                                let response = ResponseEnum::GetResponse(GetResponse {
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                    resource: Resource::Password,
                                    meta: None,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Search(request) => {
                            let response = handle_search_request(
                                request.clone(),
                                store,
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
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                    resource: request.resource,
                                    meta: None,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Fetch(request) => {
                            let response = handle_fetch_request(
                                request.clone(),
                                store,
                                passphrase_provider.clone(),
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
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                    resource: request.resource,
                                    meta: None,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Init(request) => {
                            let response = handle_init_request(request);
                            if response.is_ok() {
                                stores = response?;
                                let mut data = HashMap::new();
                                data.insert(DataFieldType::Data, json!([]));
                                let response = ResponseEnum::InitResponse(InitResponse {
                                    status: Status::Success,
                                    acknowledgement: None,
                                    data,
                                });
                                send_as_json(&response)?;
                                Ok(response)
                            } else {
                                let mut data = HashMap::new();
                                data.insert(
                                    DataFieldType::ErrorMessage,
                                    serde_json::to_value(response.unwrap_err()).unwrap(),
                                );
                                let response = ResponseEnum::InitResponse(InitResponse {
                                    status: Status::Failure,
                                    acknowledgement: None,
                                    data,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Login(request) => {
                            let store_res = handle_login_request(
                                request.clone(),
                                &stores,
                                passphrase_provider.clone(),
                            );
                            let mut data = HashMap::new();
                            if store_res.is_ok() {
                                store_opt = Some(store_res?);
                                let response = ResponseEnum::LoginResponse(LoginResponse {
                                    status: Status::Success,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
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
                                    data,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Logout(request) => {
                            let res =
                                handle_logout_request(request.clone(), store, passphrase_provider);
                            let mut data = HashMap::new();
                            if res.is_ok() {
                                store_opt = None;
                                let response = ResponseEnum::LogoutResponse(LogoutResponse {
                                    status: Status::Success,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                });
                                send_as_json(&response)?;
                                Ok(response)
                            } else {
                                data.insert(
                                    DataFieldType::ErrorMessage,
                                    serde_json::to_value(res.unwrap_err()).unwrap(),
                                );
                                let response = LogoutResponse {
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                };
                                send_as_json(&response)?;
                                Err(ResponseEnum::LogoutResponse(response))
                            }
                        }
                        RequestEnum::Create(request) => {
                            let response = handle_create_request(
                                request.clone(),
                                store,
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
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                    resource: request.resource,
                                    meta: None,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Delete(request) => {
                            let response = handle_delete_request(
                                request.clone(),
                                store,
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
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Edit(request) => {
                            let response = handle_edit_request(
                                request.clone(),
                                store,
                                passphrase_provider.clone(),
                            );
                            let mut data = HashMap::new();
                            if response.is_ok() {
                                let response = ResponseEnum::EditResponse(response?);
                                send_as_json(&response)?;
                                Ok(response)
                            } else {
                                data.insert(
                                    DataFieldType::ErrorMessage,
                                    serde_json::to_value(response.unwrap_err()).unwrap(),
                                );
                                let response = ResponseEnum::EditResponse(EditResponse {
                                    id: request.id.clone(),
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                    resource: request.resource,
                                    meta: None,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        _ => {
                            let mut data = HashMap::new();
                            data.insert(DataFieldType::ErrorMessage, json!("Unknown request"));
                            data.insert(DataFieldType::Request, json!(request));
                            Err(ResponseEnum::GenericError(GenericError {
                                status: Status::Failure,
                                acknowledgement: request.get_acknowledgement(),
                                data,
                            }))
                        }
                    }
                } else {
                    let passphrase_provider = Some(Handler::new(
                        passphrases
                            .clone()
                            .unwrap_or(Arc::new(RwLock::new(HashMap::new()))),
                        Some(request.get_type()),
                    ));
                    match request.clone() {
                        RequestEnum::Init(request) => {
                            let response = handle_init_request(request.clone());
                            let mut data = HashMap::new();
                            if response.is_ok() {
                                stores = response?;
                                let response = ResponseEnum::InitResponse(InitResponse {
                                    status: Status::Success,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                });
                                send_as_json(&response)?;
                                Ok(response)
                            } else {
                                data.insert(
                                    DataFieldType::ErrorMessage,
                                    serde_json::to_value(response.unwrap_err()).unwrap(),
                                );
                                let response = ResponseEnum::InitResponse(InitResponse {
                                    status: Status::Failure,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        RequestEnum::Login(request) => {
                            let store_res = handle_login_request(
                                request.clone(),
                                &stores,
                                passphrase_provider.clone(),
                            );
                            let mut data = HashMap::new();
                            if store_res.is_ok() {
                                store_opt = Some(store_res?);
                                let response = ResponseEnum::LoginResponse(LoginResponse {
                                    status: Status::Success,
                                    acknowledgement: request.acknowledgement.clone(),
                                    data,
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
                                    data,
                                });
                                send_as_json(&response)?;
                                Err(response)
                            }
                        }
                        _ => {
                            let mut data = HashMap::new();
                            data.insert(DataFieldType::ErrorMessage, json!("Unknown request"));
                            data.insert(DataFieldType::Request, json!(request));
                            error!("Only login request could be accepted when no store has been set. Request was: {:?}", request);
                            Err(ResponseEnum::GenericError(GenericError {
                                status: Status::Failure,
                                acknowledgement: request.get_acknowledgement(),
                                data,
                            }))
                        }
                    }
                }
            };
            if let Err(error) = request_result {
                error!("Failed to handle {:?} request. Err: {:?}", request, error);
                send_as_json(&error)?;
            }
        } else {
            error!("Failed to parse message: {:?}", received_message);
        }
    }
}
