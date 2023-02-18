use crate::cli;
use include_directory::Dir;
use static_file_http_server_macros::single_binary_producer_dir;
use std::process::Command;
use tokio::{
    fs::{self, File as TokioFile},
    io::AsyncWriteExt,
};

// The macro generates the code so `$CARGO_MANIFEST_DIR/../../../single-binary-producer` directory
// is gonna be used when running `CARGO_PUBLISH=true cargo publish` and
// `$CARGO_MANIFEST_DIR/single-binary-producer` is gonna be used when running normally.
const SINGLE_BINARY_PRODUCER_DIR: Dir<'static> = single_binary_producer_dir!();

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
async fn copy_from_single_binary_producer_to_cache(path: impl ToString) {
    write_to_cache(
        &path,
        SINGLE_BINARY_PRODUCER_DIR
            .get_file(path.to_string())
            .unwrap()
            .contents_utf8()
            .unwrap(),
    )
    .await;
}

async fn create_the_sample_project(args: &cli::Args) {
    copy_from_single_binary_producer_to_cache("Cargo.toml").await;

    let main_rs_path = "src/main.rs";
    let mut main_rs = SINGLE_BINARY_PRODUCER_DIR
        .get_file(main_rs_path)
        .unwrap()
        .contents_utf8()
        .unwrap()
        .to_string();
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
