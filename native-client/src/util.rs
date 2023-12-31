use std::{collections::HashMap, time::SystemTime};

use browser_rpass::request::DataFieldType;
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use rpass::{
    crypto::{Handler, Key},
    pass,
};
use serde_json::Value;

use crate::PasswordStoreType;

pub fn setup_logger() -> std::result::Result<(), fern::InitError> {
    let home = std::env::var("HOME").unwrap_or("".to_string());
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::BrightWhite)
        .debug(Color::BrightMagenta)
        .trace(Color::BrightBlack);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                    "{header_color}[{date} {target}][{color_line}{level}{header_color}] {color_line}{message} {footer_color}[{file}:{line_number}]\x1B[0m ",
                    header_color=
                    format_args!(
                        "\x1B[{}m",
                        colors_line.get_color(&record.level()).to_fg_str()
                        ),
                        color_line=format_args!(
                            "\x1B[{}m",
                            colors_line.get_color(&record.level()).to_fg_str()
                            ),
                            date = humantime::format_rfc3339_seconds(SystemTime::now()),
                            target = record.target(),
                            level = record.level(),
                            message = message,
                            footer_color=
                            format_args!(
                                "\x1B[{}m",
                                colors_line.get_color(&record.level()).to_fg_str()
                                ),
                                file = record.file().unwrap_or("unknown"),
                                line_number = record.line().unwrap_or(0)
                                    ));
        })
    .chain(std::io::stderr())
        .chain(
            fern::Dispatch::new().level(LevelFilter::Warn).chain(
                fern::log_file(format!(
                        "{}/rpass/browser-rpass/native-client/logs/output-{}.log",
                        home,
                        chrono::offset::Local::now()
                        ))?))
        .chain(
            fern::Dispatch::new().level(LevelFilter::Warn).chain(
                std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(format!("{}/rpass/browser-rpass/native-client/logs/output.log",home))?,
                ))
            .apply()?;
    Ok(())
}
pub fn merge_json(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
#[allow(dead_code)]
pub fn do_rename_file(
    old_name: &str,
    new_name: &str,
    store: PasswordStoreType,
    passphrase_provider: Option<Handler>,
) -> pass::Result<()> {
    let res = store
        .lock()?
        .lock()?
        .rename_file(old_name, &new_name, passphrase_provider);
    res.map(|_| ())
}

pub trait ToJson {
    fn to_json(&self) -> Value;
}
impl ToJson for dyn Key {
    fn to_json(&self) -> Value {
        let mut map = HashMap::new();
        let fingerprint = self.fingerprint().unwrap();
        let fingerprint = hex::encode(fingerprint);
        map.insert(
            DataFieldType::KeyFingerprint,
            serde_json::to_value(fingerprint.clone()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyID,
            serde_json::to_value(fingerprint).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsername,
            serde_json::to_value(self.get_user_name()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUserID,
            serde_json::to_value(self.primary_user_id()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyHasSecret,
            serde_json::to_value(self.has_secret()).unwrap(),
        );
        map.insert(
            DataFieldType::KeyUsable,
            serde_json::to_value(!self.is_not_usable()).unwrap(),
        );
        map.insert(
            DataFieldType::CanSign,
            serde_json::to_value(self.can_sign()).unwrap(),
        );
        map.insert(
            DataFieldType::CanEncrypt,
            serde_json::to_value(self.can_encrypt()).unwrap(),
        );
        serde_json::to_value(map).unwrap()
    }
}
