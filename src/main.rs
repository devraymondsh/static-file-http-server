use once_cell::sync::Lazy;

pub(crate) mod cli;
mod compile;
mod server;

#[tokio::main]
async fn main() {
    static ARGS: Lazy<cli::Args> = Lazy::new(|| cli::parse());

    if ARGS.single_binary {
        compile::compile(&ARGS).await;
    } else {
        server::run(&ARGS).await;
    }
}
