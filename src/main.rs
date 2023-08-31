mod cli;
mod config;
mod db;
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

    let (db, db_serv) = match db::make(&conf.db) {
        Ok(db) => db,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let db_handle = tokio::spawn(db_serv.run());
    let server_handle = tokio::spawn(server::run(conf.net, db));
    _ = tokio::join!(db_handle, server_handle);
}
