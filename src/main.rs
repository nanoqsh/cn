mod cache;
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
            eprintln!("config error: {err}");
            return;
        }
    };

    let (db, db_serv) = match db::make(&conf.db) {
        Ok(db) => db,
        Err(err) => {
            eprintln!("database error: {err}");
            return;
        }
    };

    let db_handle = tokio::spawn(db_serv.run());
    let server_handle = tokio::spawn(server::run(conf.net, db));
    tokio::select! {
        Ok(Err(err)) = db_handle => eprintln!("database error: {err}"),
        _ = server_handle => {}
    }
}
