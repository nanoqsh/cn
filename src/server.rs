use {
    crate::{config::Net, db::Access},
    axum::{
        extract::Path,
        http::StatusCode,
        response::{Redirect, Result},
        routing, Router, Server,
    },
};

pub async fn run(conf: Net, db: Access) {
    let app = Router::new()
        .route(
            "/store/:key",
            routing::post({
                let db = db.clone();
                |Path(key), link| store(db, key, link)
            }),
        )
        .route("/:key", routing::get(|Path(key)| load(db, key)));

    let addr = conf.socket_addr();
    println!("listening on http://{addr}");

    let service = app.into_make_service();
    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("error: {err}");
    }
}

async fn store(db: Access, key: String, link: String) {
    db.store(key, link).await;
}

async fn load(db: Access, key: String) -> Result<Redirect> {
    let link = db.load(key).await.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Redirect::temporary(&link))
}
