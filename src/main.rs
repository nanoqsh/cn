mod cli;
mod config;
mod server;

#[tokio::main]
async fn main() {
    use {
        crate::{cli::Cli, config::Config, server},
        clap::Parser,
    };

    let cli = Cli::parse();
    let conf = match Config::load(cli.conf_path()) {
        Ok(conf) => conf,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    server::run(conf.net).await;
}
