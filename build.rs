// This file only downloads required files if they don't exist when running `cargo install`.

use std::{env, path::PathBuf};
use tokio::fs;

const PARENT_URL: &str = "https://raw.githubusercontent.com/devraymondsh/static-file-http-server";

async fn download(url: String) -> String {
    match reqwest::get(url).await {
        Ok(resp) => String::from_utf8_lossy(&resp.bytes().await.unwrap()).to_string(),
        Err(err) => panic!("Failed to download! Reason: {}", err.to_string()),
    }
}
fn get_project_dir() -> PathBuf {
    let mut current_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let i = if env::var("CARGO_PUBLISH").is_err() {
        5
    } else {
        7
    };
    for _ in 0..i {
        current_dir.pop();
    }

    current_dir
}

// sbp = single binary producer
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let parent_url = format!(
        "{}/{}/single-binary-producer",
        PARENT_URL,
        env::var("CARGO_PKG_VERSION").unwrap()
    );

    let project_dir = get_project_dir();
    let sbp_main_rs = PathBuf::from(format!(
        "{}/single-binary-producer/src/main.rs",
        project_dir.to_str().unwrap()
    ));
    let sbp_cargo_toml = PathBuf::from(format!(
        "{}/single-binary-producer/Cargo.toml",
        project_dir.to_str().unwrap()
    ));

    if !sbp_main_rs.exists() {
        let mut folder = sbp_main_rs.clone();
        folder.pop();

        fs::create_dir_all(folder).await.unwrap();

        let main_rs_url = format!("{}/src/main.rs", parent_url);
        let main_rs_contents = download(main_rs_url).await;

        fs::write(sbp_main_rs, main_rs_contents).await.unwrap();
    }
    if !sbp_cargo_toml.exists() {
        let mut folder = sbp_cargo_toml.clone();
        folder.pop();

        fs::create_dir_all(folder).await.unwrap();

        let cargo_toml_url = format!("{}/Cargo.toml", parent_url);
        let cargo_toml_contents = download(cargo_toml_url).await;

        fs::write(sbp_cargo_toml, cargo_toml_contents)
            .await
            .unwrap();
    }
}
