use browser_rpass::request::*;
use browser_rpass::response::InitResponse;
use browser_rpass::response::ResponseEnum;
use browser_rpass::response::Status;
use log::*;
use native_client::request_handler::*;
use native_client::util::*;

use rpass::crypto::Handler;
use rpass::pass::{self, Error};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::{thread, time};
fn main() -> pass::Result<()> {
    if let Err(log_init_error) = setup_logger() {
        eprintln!("Failed to initialize logger: {:?}", log_init_error);
    }
    trace!("Starting rpass");
    let received_message_res = get_message();
    if let Err(e) = received_message_res {
        error!("{err_msg}", err_msg = e);
        return Err(e);
    }
    let received_message = received_message_res.unwrap();
    if let Ok(request) = serde_json::from_value::<RequestEnum>(received_message.clone()) {
        match request {
            RequestEnum::Init(request) => {
                let stores;
                let acknowledgement = request.acknowledgement.clone();
                let response = handle_init_request(request);
                if response.is_ok() {
                    stores = response?;
                    let mut data = HashMap::new();
                    let mut store_ids = Vec::new();
                    for store in stores.clone().lock().unwrap().iter() {
                        let locked = store.lock().unwrap();
                        store_ids.push(locked.get_name().clone());
                    }
                    data.insert(DataFieldType::Data, serde_json::to_value(store_ids)?);
                    let response = ResponseEnum::InitResponse(InitResponse {
                        status: Status::Success,
                        acknowledgement,
                        data,
                    });
                    send_as_json(&response)?;
                    let passphrases = Arc::new(RwLock::new(HashMap::new()));
                    thread::sleep(time::Duration::from_millis(200));
                    let passphrase_provider = Some(Handler::new(passphrases));
                    listen_to_native_messaging(stores, passphrase_provider)
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
                    Err(Error::GenericDyn(format!(
                        "Failed to initialize: {:?}",
                        response
                    )))
                }
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
