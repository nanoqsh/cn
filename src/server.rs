use {
    crate::{config::Net, db::Access},
    axum::{
        extract::Path,
        http::StatusCode,
        response::{Redirect, Result},
        routing::get,
        Router, Server,
    },
};

pub async fn run(conf: Net, db: Access) {
    let app = Router::new().route("/:key", get(|Path(key)| load(key, db)));
    let addr = conf.socket_addr();
    println!("listening on http://{addr}");

    let service = app.into_make_service();
    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("error: {err}");
    }
}

async fn load(key: String, db: Access) -> Result<Redirect> {
    let link = db.load(key).await.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Redirect::temporary(&link))
}
