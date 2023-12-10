use std::{collections::HashMap, path::PathBuf};

#[allow(unused_imports)]
use log::*;
use rpass::pass::{self, PasswordEntry};

use crate::PasswordStoreType;
/// Validates the config for password stores.
/// Returns a list of paths that the new store wizard should be run for
fn _validate_stores_config(
    settings: &config::Config,
    home: &Option<PathBuf>,
) -> pass::Result<Vec<PathBuf>> {
    let mut incomplete_stores: Vec<PathBuf> = vec![];
    let stores_res = settings.get("stores");
    if let Ok(stores) = stores_res {
        let stores: HashMap<String, config::Value> = stores;

        for store_name in stores.keys() {
            let store: HashMap<String, config::Value> =
                stores.get(store_name).ok_or(pass::Error::GenericDyn(format!("store name: {store_name} exists in the passed config file: {settings:?}, but not in the store list: {stores:?}",store_name=store_name,settings=settings,stores=stores)))?.clone().into_table()?;

            let password_store_dir_opt = store.get("path");

            if let Some(p) = password_store_dir_opt {
                let p_path = PathBuf::from(p.clone().into_str()?);
                let gpg_id = p_path.clone().join(".gpg-id");

                if !p_path.exists() || !gpg_id.exists() {
                    incomplete_stores.push(PathBuf::from(p.clone().into_str()?));
                }
            }
        }
    } else if incomplete_stores.is_empty() && home.is_some() {
        incomplete_stores.push(
            home.clone()
                .ok_or(pass::Error::GenericDyn(format!(
                    "no stores specified in config file: {settings:?} and home dir is none",
                    settings = settings
                )))?
                .join(".password_store"),
        );
    }

    Ok(incomplete_stores)
}
pub fn filter_entries(store: &PasswordStoreType, query: &str) -> pass::Result<Vec<PasswordEntry>> {
    let first_locked = store.lock()?;
    let locked_store = first_locked.lock()?;
    let passwords = &*locked_store.passwords;
    pub fn normalized(s: &str) -> String {
        s.to_lowercase()
    }
    pub fn matches(s: &str, q: &str) -> bool {
        normalized(s).as_str().contains(normalized(q).as_str())
    }
    let matching = passwords
        .iter()
        .filter(|p| matches(&p.id.to_string(), query));
    let result = matching.cloned().collect();
    Ok(result)
}
