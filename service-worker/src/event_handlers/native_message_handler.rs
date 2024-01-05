use crate::store::SessionAction;
use crate::store::SessionActionWrapper;
use crate::store::SessionStore;
use crate::store::REQUEST_MAP;
use crate::Resource;
use browser_rpass::js_binding::extension_api::Port;
use log::*;
pub use wasm_bindgen;
pub use wasm_bindgen_futures;
use yewdux::dispatch::Dispatch;

use browser_rpass::request::*;
use browser_rpass::response::*;
use serde_json;
use serde_json::json;
use serde_json::Value;

pub fn process_native_message(
    json_msg: Value,
    native_port: Option<&Port>,
    ctx: Option<Value>,
) -> Result<ResponseEnum, String> {
    let session_store_dispatch = Dispatch::<SessionStore>::new();
    let response_wrapper = serde_json::from_value::<ResponseEnum>(json_msg.clone()).unwrap();
    let acknowledgement = response_wrapper.get_acknowledgement();
    let request = if let Some(ref acknowledgement) = acknowledgement {
        REQUEST_MAP
            .lock()
            .unwrap()
            .get(acknowledgement)
            .map(|req| req.clone())
    } else {
        None
    };
    match response_wrapper {
        ResponseEnum::LoginResponse(login_response) => {
            if let RequestEnum::Login(login_request) = request.clone().unwrap() {
                let login_response2 = login_response.clone();
                let mut ctx = ctx.unwrap_or(json!({}));
                ctx["store_id"] = json!(login_request.store_id);
                ctx["is_default"] = json!(login_request.is_default);
                ctx["acknowledgement"] = json!(login_request.acknowledgement);
                wasm_bindgen_futures::spawn_local(async move {
                    let login_response = login_response2;
                    match login_response.status {
                        Status::Success => {
                            session_store_dispatch.apply(SessionActionWrapper {
                                action: SessionAction::Login,
                                meta: Some(ctx),
                            });
                        }
                        Status::Failure => session_store_dispatch.apply(SessionActionWrapper {
                            action: SessionAction::LoginError(login_request.clone()),
                            meta: Some(ctx),
                        }),
                        _ => {}
                    };
                });
                let response = ResponseEnum::LoginResponse(login_response);
                return Ok(response);
            } else {
                return Err(
                    "response is for login request but request type is not login".to_owned(),
                );
            }
        }
        ResponseEnum::LogoutResponse(logout_response) => {
            let logout_response2 = logout_response.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let logout_response = logout_response2;
                match logout_response.status {
                    Status::Success => {
                        session_store_dispatch.apply(SessionActionWrapper {
                            action: SessionAction::Logout(
                                logout_response.store_id,
                                logout_response.acknowledgement,
                            ),
                            meta: None,
                        });
                    }
                    Status::Failure => session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::LogoutError(logout_response.clone()),
                        meta: None,
                    }),
                    _ => {}
                };
            });
            let response = ResponseEnum::LogoutResponse(logout_response);
            return Ok(response);
        }
        ResponseEnum::GetResponse(get_response) => {
            let response = ResponseEnum::GetResponse(get_response);
            return Ok(response);
        }
        ResponseEnum::DeleteResponse(delete_response) => {
            match delete_response.status.clone() {
                Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::DataDeleted(
                            Resource::Account,
                            delete_response.deleted_resource_id.clone(),
                            delete_response.data.clone(),
                        ),
                        meta: ctx,
                    });
                }
                _ => {}
            }
            let response = ResponseEnum::DeleteResponse(delete_response);
            return Ok(response);
        }
        ResponseEnum::CreateResponse(create_response) => {
            let response = ResponseEnum::CreateResponse(create_response.clone());
            let status = &create_response.status;
            match status {
                &Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::DataCreated(create_response),
                        meta: ctx,
                    });
                }
                _ => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::DataCreationFailed(
                            create_response.resource.clone(),
                            create_response.data.clone(),
                            request,
                        ),
                    });
                }
            }
            return Ok(response);
        }
        ResponseEnum::CreateStoreResponse(create_store_response) => {
            let response = ResponseEnum::CreateStoreResponse(create_store_response.clone());
            let status = &create_store_response.status;
            match status {
                &Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::StoreCreated(create_store_response),
                        meta: ctx,
                    });
                }
                _ => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::StoreCreationFailed(
                            request.clone().unwrap(),
                            response.clone(),
                        ),
                    });
                }
            }
            return Ok(response);
        }
        ResponseEnum::DeleteStoreResponse(delete_store_response) => {
            let response = ResponseEnum::DeleteStoreResponse(delete_store_response.clone());
            let status = &delete_store_response.status;
            match status {
                &Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::StoreDeleted(delete_store_response),
                        meta: ctx,
                    });
                }
                _ => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::StoreDeletionFailed(
                            request.clone().unwrap(),
                            response.clone(),
                        ),
                    });
                }
            }
            return Ok(response);
        }
        ResponseEnum::EditResponse(edit_response) => {
            let response = ResponseEnum::EditResponse(edit_response.clone());
            let status = &edit_response.status;
            match status {
                &Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        action: SessionAction::DataEdited(edit_response),
                        meta: ctx,
                    });
                }
                _ => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::DataEditFailed(
                            edit_response.resource.clone(),
                            edit_response.data.clone(),
                            request,
                        ),
                    });
                }
            }
            return Ok(response);
        }
        ResponseEnum::SearchResponse(search_response) => {
            let response = ResponseEnum::SearchResponse(search_response);
            return Ok(response);
        }
        ResponseEnum::FetchResponse(fetch_response) => {
            let response = ResponseEnum::FetchResponse(fetch_response.clone());
            match fetch_response.status.clone() {
                Status::Success => {
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::DataFetched(fetch_response),
                    });
                    return Ok(response);
                }
                _ => {
                    return Err("error happened while fetching data".to_owned());
                }
            }
        }
        ResponseEnum::InitResponse(init_response) => {
            let status = &init_response.status;
            match status {
                Status::Success => {
                    let response = ResponseEnum::InitResponse(init_response.clone());
                    session_store_dispatch.apply(SessionActionWrapper {
                        meta: ctx,
                        action: SessionAction::Init(init_response),
                    });
                    return Ok(response);
                }
                _ => {
                    error!("error happened while initializing");
                    return Err("error happened while initializing".to_owned());
                }
            }
        }
        _ => {
            let error_response = ErrorResponse {
                message: Some("resource not supported".to_owned()),
                acknowledgement,
                code: Some(ErrorCode::NotSupported),
            };
            return Ok(ResponseEnum::ErrorResponse(error_response));
        }
    };
}
