use log::*;
use native_client::request_handler::*;
use native_client::util::*;
use native_client::StoreListType;

use rpass::crypto::Handler;
use rpass::pass::PasswordStore;
use rpass::pass::{self};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
fn main() -> pass::Result<()> {
    if let Err(log_init_error) = setup_logger() {
        eprintln!("Failed to initialize logger: {:?}", log_init_error);
    }
    trace!("Starting rpass");
    let passphrases = Arc::new(RwLock::new(HashMap::new()));
    let passphrase_provider = Some(Handler::new(passphrases));
    let mut home = {
        match std::env::var("HOME") {
            Err(_) => None,
            Ok(home_path) => Some(PathBuf::from(home_path)),
        }
    };
    let password_store_dir = {
        match std::env::var("PASSWORD_STORE_DIR") {
            Err(_) => None,
            Ok(password_store_dir) => Some(password_store_dir),
        }
    };
    let password_store_signing_key = {
        match std::env::var("PASSWORD_STORE_SIGNING_KEY") {
            Err(_) => None,
            Ok(password_store_signing_key) => Some(password_store_signing_key),
        }
    };
    if home.is_none() {
        home = Some(match std::env::var("XDG_DATA_HOME") {
            Err(_) => match &home {
                Some(home_path) => home_path.join(".local"),
                None => {
                    return Err(pass::Error::from("No home directory set"));
                }
            },
            Ok(data_home_path) => PathBuf::from(data_home_path),
        })
    }

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
    if let Ok(stores) = PasswordStore::get_stores(&config, &home) {
        let stores: StoreListType = Arc::new(Mutex::new(
            stores
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
        listen_to_native_messaging(stores, passphrase_provider, home, config_file_location)
    } else {
        let stores: StoreListType = Arc::new(Mutex::new(Vec::new()));
        listen_to_native_messaging(stores, passphrase_provider, home, config_file_location)
    }
}
