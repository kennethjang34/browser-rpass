use std::fs;
use std::process::Command;

fn main() {
    // Remove existing pkg directory
    let _ = fs::remove_dir_all("./pkg");

    // Create pkg directory
    fs::create_dir("./pkg").expect("Unable to create pkg directory");
    // trunk build for popup/index.html
    Command::new("trunk")
        .arg("build")
        .arg("./popup/index.html")
        .arg("--dist")
        .arg("./pkg")
        .output()
        .expect("Failed to build popup/index.html");

    // Cargo build for native-client
    Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("native-client")
        .arg("--target-dir")
        .arg("../pkg")
        .output()
        .expect("Failed to build native-client");

    // wasm-pack build for service-worker
    Command::new("wasm-pack")
        .args(&[
            "build",
            "./service-worker",
            "--target",
            "web",
            "--out-dir",
            "../pkg",
            "--dev",
        ])
        .output()
        .expect("Failed to build service-worker");

    // wasm-pack build for content
    Command::new("wasm-pack")
        .args(&[
            "build",
            "./content",
            "--target",
            "web",
            "--out-dir",
            "../pkg",
            "--dev",
        ])
        .output()
        .expect("Failed to build content");

    // Copy necessary files to pkg directory
    //fs::create_dir_all("./pkg").expect("Unable to create pkg directory");
    fs::copy("./init_popup.js", "./pkg").expect("Failed to copy init_popup.js");
    fs::copy("./run_service_worker.js", "./pkg").expect("Failed to copy run_service_worker.js");
    fs::copy("./run_content.js", "./pkg").expect("Failed to copy run_content.js");
    fs::copy("./manifest_v3.json", "./pkg/manifest.json").expect("Failed to copy manifest_v3.json");

    // Check if styles.css and popup_styles.css exist and update if necessary
    if !fs::metadata("./popup/assets/styles.css").is_ok() {
        println!("styles.css does not exist");
        std::process::exit(1);
    }

    if !fs::metadata("./assets/popup_styles.css").is_ok() {
        println!("popup_styles.css does not exist. Creating it.");
        Command::new("npx")
            .args(&[
                "tailwindcss",
                "-i",
                "./popup/assets/styles.css",
                "-o",
                "./assets/popup_styles.css",
            ])
            .output()
            .expect("Failed to create popup_styles.css");
        std::process::exit(1);
    }

    // Compare modification times for styles.css and popup_styles.css
    let styles_css_mod = fs::metadata("./popup/assets/styles.css")
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap()
        .as_secs();
    let popup_styles_css_mod = fs::metadata("./assets/popup_styles.css")
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap()
        .as_secs();

    // Execute npx tailwindcss if styles.css is newer than popup_styles.css
    if styles_css_mod > popup_styles_css_mod {
        fs::remove_file("./assets/popup_styles.css").expect("Failed to remove popup_styles.css");
        Command::new("npx")
            .args(&[
                "tailwindcss",
                "-i",
                "./popup/assets/styles.css",
                "-o",
                "./assets/popup_styles.css",
            ])
            .output()
            .expect("Failed to update popup_styles.css");
        println!("Changes detected. TailwindCSS has been executed.");
    } else {
        println!("No changes detected.");
    }

    // Check if styles.css and content_styles.css exist and update if necessary
    if !fs::metadata("./content/assets/styles.css").is_ok() {
        println!("styles.css does not exist");
        std::process::exit(1);
    }

    if !fs::metadata("./assets/content_styles.css").is_ok() {
        Command::new("npx")
            .args(&[
                "tailwindcss",
                "-i",
                "./content/assets/styles.css",
                "-o",
                "./assets/content_styles.css",
            ])
            .output()
            .expect("Failed to create content_styles.css");
        std::process::exit(1);
    }

    // Compare modification times for styles.css and content_styles.css
    let styles_css_mod = fs::metadata("./content/assets/styles.css")
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap()
        .as_secs();
    let content_styles_css_mod = fs::metadata("./assets/content_styles.css")
        .unwrap()
        .modified()
        .unwrap()
        .elapsed()
        .unwrap()
        .as_secs();

    // Execute npx tailwindcss if styles.css is newer than content_styles.css
    if styles_css_mod > content_styles_css_mod {
        fs::remove_file("./assets/content_styles.css")
            .expect("Failed to remove content_styles.css");
        Command::new("npx")
            .args(&[
                "tailwindcss",
                "-i",
                "./content/assets/styles.css",
                "-o",
                "./assets/content_styles.css",
            ])
            .output()
            .expect("Failed to update content_styles.css");
        println!("TailwindCSS has been executed.");
    } else {
        println!("No changes detected.");
    }

    // Copy all files from assets directory to pkg directory
    fs::copy("./assets/", "./pkg").expect("Failed to copy assets to pkg");
}
