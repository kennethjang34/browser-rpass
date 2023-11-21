use crate::store::SessionAction;
use crate::store::SessionActionWrapper;
use crate::store::SessionStore;
use crate::Resource;
use browser_rpass::dbg;
use browser_rpass::util::*;
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
    _native_port: Port,
    request: Option<RequestEnum>,
    ctx: Option<Value>,
) -> Result<ResponseEnum, String> {
    let session_store_dispatch = Dispatch::<SessionStore>::new();
    if let Some(request) = request {
        match request {
            RequestEnum::Login(login_request) => {
                let login_response: LoginResponse =
                    serde_json::from_value::<LoginResponse>(json_msg).unwrap();
                let login_response2 = login_response.clone();
                let ctx = ctx.map_or(json!({"passphrase":login_request.passphrase}), |mut ctx| {
                    ctx["passphrase"] = json!(login_request.passphrase);
                    ctx["user_id"] = json!(login_request.username);
                    ctx["acknowledgement"] = json!(login_request.acknowledgement);
                    ctx
                });
                wasm_bindgen_futures::spawn_local(async move {
                    let login_response = login_response2;
                    match login_response.status {
                        Status::Success => {
                            dbg!(&login_response);
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
            }
            RequestEnum::Logout(logout_request) => {
                let logout_response: LogoutResponse =
                    serde_json::from_value::<LogoutResponse>(json_msg).unwrap();
                let logout_response2 = logout_response.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let logout_response = logout_response2;
                    match logout_response.status {
                        Status::Success => {
                            dbg!(&logout_response);
                            session_store_dispatch.apply(SessionActionWrapper {
                                action: SessionAction::Logout,
                                meta: None,
                            });
                        }
                        Status::Failure => session_store_dispatch.apply(SessionActionWrapper {
                            action: SessionAction::LogoutError(logout_request.clone()),
                            meta: None,
                        }),
                        _ => {}
                    };
                });
                let response = ResponseEnum::LogoutResponse(logout_response);
                return Ok(response);
            }
            RequestEnum::Get(_get_request) => {
                let get_response: GetResponse =
                    serde_json::from_value::<GetResponse>(json_msg).unwrap();
                let response = ResponseEnum::GetResponse(get_response);
                return Ok(response);
            }
            RequestEnum::Delete(_delete_request) => {
                let delete_response: DeleteResponse =
                    serde_json::from_value::<DeleteResponse>(json_msg).unwrap();
                match delete_response.status.clone() {
                    Status::Success => {
                        session_store_dispatch.apply(SessionActionWrapper {
                            action: SessionAction::DataDeleted(
                                Resource::Account,
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
            RequestEnum::Create(ref create_request) => {
                dbg!(&json_msg);
                let create_response: CreateResponse =
                    serde_json::from_value::<CreateResponse>(json_msg).unwrap();
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
                                create_request.resource.clone(),
                                create_response.data.clone(),
                                Some(request.clone()),
                            ),
                        });
                    }
                }
                return Ok(response);
            }
            RequestEnum::Edit(ref edit_request) => {
                dbg!(&json_msg);
                let edit_response: EditResponse =
                    serde_json::from_value::<EditResponse>(json_msg).unwrap();
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
                                edit_request.resource.clone(),
                                edit_response.data.clone(),
                                Some(request.clone()),
                            ),
                        });
                    }
                }
                return Ok(response);
            }
            RequestEnum::Search(ref _search_request) => {
                let search_response: SearchResponse =
                    serde_json::from_value::<SearchResponse>(json_msg).unwrap();
                let response = ResponseEnum::SearchResponse(search_response);
                return Ok(response);
            }
            RequestEnum::Fetch(ref _fetch_request) => {
                let fetch_response: FetchResponse =
                    serde_json::from_value::<FetchResponse>(json_msg).unwrap();
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
            RequestEnum::Init(ref _init_request) => {
                let init_response: InitResponse =
                    serde_json::from_value::<InitResponse>(json_msg).unwrap();
                let response = ResponseEnum::InitResponse(init_response);
                return Ok(response);
            }
            _ => {
                let error_response = ErrorResponse {
                    message: Some("resource not supported".to_owned()),
                    acknowledgement: request.get_acknowledgement(),
                    code: Some(ErrorCode::NotSupported),
                };
                return Ok(ResponseEnum::ErrorResponse(error_response));
            }
        };
    } else {
        todo!()
    }
}
