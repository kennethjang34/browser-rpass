use std::process::Command;
use std::{env, str};

fn main() {
    let pineentry_path = std::option_env!("PINENTRY_PATH");
    if let Some(pinentry_path) = pineentry_path {
        println!("cargo:rustc-env=PINENTRY_PATH={}", pinentry_path);
    } else {
        let current_os = env::consts::OS;
        let pinentry_path = {
            match current_os {
                "macos" => Command::new("which").arg("pinentry-mac").output().ok(),
                "linux" => Command::new("which").arg("pinentry").output().ok(),
                _ => None,
            }
        };
        if let Some(pinentry_path) = pinentry_path {
            unsafe {
                let pinentry_path = str::from_utf8_unchecked(&(pinentry_path).stdout);
                println!("cargo:rustc-env=PINENTRY_PATH={}", pinentry_path);
            }
        }
    }
}
