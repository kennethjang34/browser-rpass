use browser_rpass::request::*;
use log::*;
use native_client::request_handler::*;
use native_client::util::*;

use rpass::pass::{self, Error};
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
                let stores = handle_init_request(request)?;
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
