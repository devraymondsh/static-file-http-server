use clap::{
    builder,
    error::{Error as ClapError, ErrorKind},
    Command as ClapCommand, Parser,
};
use std::{
    fmt::Display,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    process,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the directory.
    pub path: PathBuf,

    /// Address to bind, for example: 0.0.0.0:80.
    /// You may need administrator permissions for binding on port 80 based on your OS.
    #[arg(short, long, default_value_t = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 80))]
    pub addr: SocketAddr,

    /// Controling CORS via the 'Access-Control-Allow-Origin' header.
    #[arg(short = 'r', long, default_value_t = String::from("*"))]
    pub cors: String,

    /// Produce a single binary that serves files that get embedded in the binary for better performance. You need to install Rust and Cargo before running this feature. (Recommended for production)
    #[arg(short = 'p', long, default_value_t = false)]
    pub single_binary: bool,

    /// Open the browser after starting the server.
    #[arg(short, long, default_value_t = false)]
    pub open: bool,

    /// Set cache time (in seconds) for cache-control max-age header, for eaxmple: -c10 for 10 seconds. Use -c-1 to disable caching.
    #[arg(short, long, default_value_t = 3600)]
    pub cache: i64,
}

pub fn unrecoverable_clap_error(cmd: &builder::Command, message: impl Display) {
    let a: ClapError = ClapError::raw(ErrorKind::Io, message).with_cmd(cmd);

    let _ = a.print();

    process::exit(10);
}
pub fn unrecoverable_clap_error_with_cmd(message: impl Display) {
    let cmd = ClapCommand::new("static-file-http-server");

    unrecoverable_clap_error(&cmd, message);
}

pub fn parse() -> Args {
    let args = Args::parse();
    let cmd = ClapCommand::new("static-file-http-server");

    let Args {
        path: serve_path, ..
    } = &args;
    if !serve_path.as_path().exists() {
        let _ = unrecoverable_clap_error(
            &cmd,
            "The entered path doesn't exist. Invalid directory to serve!",
        );
    }
    if !serve_path.as_path().is_dir() {
        let _ = unrecoverable_clap_error(
            &cmd,
            "The entered path is not a directory. Invalid directory to serve!",
        );
    }

    args
}
