use crate::cli;
use static_file_http_server_macros::{get_sbp_cargo_toml, get_sbp_main_rs};
use std::process::Command;
use tokio::{
    fs::{self, File as TokioFile},
    io::AsyncWriteExt,
};

get_sbp_main_rs!("SBP_MAIN_RS");
get_sbp_cargo_toml!("SBP_CARGO_TOML");

async fn write_to_cache(path: &impl ToString, content: impl ToString) {
    let mut main_rs = TokioFile::create(format!(
        "./.static-file-http-server-cache/{}",
        path.to_string()
    ))
    .await
    .unwrap();

    main_rs
        .write_all(content.to_string().as_bytes())
        .await
        .unwrap();
}

async fn create_the_sample_project(args: &cli::Args) {
    write_to_cache(&"Cargo.toml", SBP_CARGO_TOML).await;

    let main_rs_path = "src/main.rs";
    let mut main_rs = SBP_MAIN_RS.to_string();
    main_rs = main_rs.replace("127.0.0.1:8080", args.addr.to_string().as_str());
    main_rs = main_rs.replace("3600", args.cache.to_string().as_str());

    let absolute_path = fs::canonicalize(&args.path).await.unwrap();
    main_rs = main_rs.replace("./_public_dir_", absolute_path.to_str().unwrap());

    // Because two '*' characters are used in the file
    main_rs = main_rs.replace(
        "Lazy::new(|| String::from(\"*\"))",
        format!("Lazy::new(|| String::from(\"{}\"))", args.cors).as_str(),
    );
    write_to_cache(&main_rs_path, main_rs).await;
}

fn command_exists_or_exit() {
    match Command::new("cargo").arg("--version").status() {
        Ok(status) => {
            if !status.success() {
                cli::unrecoverable_clap_error_with_cmd("`cargo` was not found! Check your PATH!")
            }
        }
        Err(err) => {
            cli::unrecoverable_clap_error_with_cmd(format!(
                "`cargo` was not found! Check your PATH! Explain: {}.",
                err.to_string(),
            ));
        }
    };
}
async fn run_cargo(current_dir: impl ToString) {
    command_exists_or_exit();

    println!("Compiling... This can take a while based on your hardware and project size.");
    let _ = Command::new("cargo")
        .env("RUSTFLAGS", "-C target-cpu=native")
        .args(["clean", "-p", "single-binary-producer"])
        .current_dir(current_dir.to_string())
        .output()
        .unwrap();
    let _ = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(current_dir.to_string())
        .output()
        .unwrap();
}

pub async fn compile(args: &cli::Args) {
    if let Err(err) = fs::create_dir_all("./.static-file-http-server-cache/src").await {
        cli::unrecoverable_clap_error_with_cmd(format!(
            "Failed to create the cache directory! Reason: {}.",
            err.to_string()
        ));
    }

    create_the_sample_project(args).await;
    run_cargo("./.static-file-http-server-cache").await;

    fs::copy(
        "./.static-file-http-server-cache/target/release/single-binary-producer",
        "./static-file-http-server",
    )
    .await
    .unwrap();

    println!(
        "Compilation finished! Use your binary to start the server by running: `./static-file-http-server`."
    );
}
