// The only purpose of this build.rs is to copy contents from the single-binary-producer directory
// to the target/package directory so it it can be embedded in the binary when compiling for publish
// by running the command `cargo publish`. So don't worry about this file as it doesn't serve any
// purpose other than preparing for publish.

use std::{env, fs, path::PathBuf};

fn get_target_dir_path() -> PathBuf {
    let mut target_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    for _ in 0..4 {
        target_dir.pop();
    }

    target_dir
}
fn get_project_dir_path() -> PathBuf {
    let mut project_dir = get_target_dir_path();
    project_dir.pop();

    project_dir
}

fn copy_files(crate_folder_name: impl ToString) {
    let mut sbp_new_main_rs = PathBuf::from(format!(
        "{}/packge/{}/single-binary-producer/src",
        get_target_dir_path().to_str().unwrap(),
        crate_folder_name.to_string()
    ));
    let mut sbp_main_rs = get_project_dir_path();
    sbp_main_rs.push("single-binary-producer");
    sbp_main_rs.push("src");
    sbp_main_rs.push("main.rs");
    fs::create_dir_all(&sbp_new_main_rs).unwrap();
    sbp_new_main_rs.push("main.rs");
    fs::copy(&sbp_main_rs, &sbp_new_main_rs).unwrap();

    let mut sbp_cargo_toml = sbp_main_rs;
    sbp_cargo_toml.pop();
    sbp_cargo_toml.pop();
    sbp_cargo_toml.push("Cargo.toml");
    let mut sbp_new_cargo_toml = sbp_new_main_rs;
    sbp_new_cargo_toml.pop();
    sbp_new_cargo_toml.pop();
    sbp_new_cargo_toml.push("Cargo.toml");
    fs::copy(&sbp_cargo_toml, &sbp_new_cargo_toml).unwrap();
}

// sbp = single binary producer
fn main() {
    let pkg_name = env::var("CARGO_PKG_NAME").unwrap();
    let pkg_version = env::var("CARGO_PKG_VERSION").unwrap();
    let crate_folder_name = format!("{}-{}", pkg_name, pkg_version);

    copy_files(crate_folder_name);
}
