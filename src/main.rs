mod config;
mod server;

#[tokio::main]
async fn main() {
    use crate::{config::Config, server};

    let conf = match Config::load("config.toml") {
        Ok(conf) => conf,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    server::run(conf).await;
}
