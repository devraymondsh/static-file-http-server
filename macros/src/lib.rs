extern crate proc_macro;

use proc_macro::{TokenStream, TokenTree};
use std::{env, fmt::Display, fs, path::PathBuf};

fn git_sbp_url(sub_uri: &impl ToString) -> String {
    format!(
        "https://raw.githubusercontent.com/devraymondsh/static-file-http-server/v{}/single-binary-producer/{}",
        env::var("CARGO_PKG_VERSION").unwrap(),
        sub_uri.to_string()
    )
}
fn download_from_repo(sub_uri: &(impl ToString + Display)) -> String {
    String::from_utf8_lossy(
        &reqwest::blocking::get(git_sbp_url(sub_uri))
            .unwrap()
            .bytes()
            .unwrap(),
    )
    .to_string()
}

const MAIN_RS_ORIGINAL_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../single-binary-producer/src/main.rs"
);
const CARGO_TOML_ORIGINAL_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../single-binary-producer/Cargo.toml"
);

fn get_main_rs_path() -> String {
    if PathBuf::from(MAIN_RS_ORIGINAL_PATH).exists() {
        let main_rs_contents = fs::read_to_string(MAIN_RS_ORIGINAL_PATH).unwrap();
        main_rs_contents
    } else {
        download_from_repo(&"src/main.rs")
    }
}
fn get_cargo_toml_path() -> String {
    if PathBuf::from(CARGO_TOML_ORIGINAL_PATH).exists() {
        let cargo_toml_contents = fs::read_to_string(CARGO_TOML_ORIGINAL_PATH).unwrap();
        cargo_toml_contents
    } else {
        download_from_repo(&"Cargo.toml")
    }
}
fn quote(name: impl ToString, contents: impl ToString) -> TokenStream {
    format!(
        "const {}: &str = r#\"{}\"#;",
        name.to_string(),
        contents.to_string()
    )
    .parse()
    .unwrap()
}

fn parse_input(input: TokenStream) -> String {
    match &input.into_iter().collect::<Vec<_>>()[0] {
        TokenTree::Literal(lit) => {
            let mut repr = lit.to_string();
            if !repr.starts_with('"') || !repr.ends_with('"') {
                panic!("This macro only accepts a single, non-empty string argument")
            }

            repr.remove(0);
            repr.pop();

            repr
        }
        _ => panic!("This macro only accepts a single, non-empty string argument"),
    }
}

#[proc_macro]
pub fn get_sbp_main_rs(input: TokenStream) -> TokenStream {
    let contents = get_main_rs_path();
    let name = parse_input(input);

    quote(name, contents)
}

#[proc_macro]
pub fn get_sbp_cargo_toml(input: TokenStream) -> TokenStream {
    let contents = get_cargo_toml_path();
    let name = parse_input(input);

    quote(name, contents)
}
