use {
    crate::config::Net,
    axum::{
        extract::Path,
        http::StatusCode,
        response::{Redirect, Result},
        routing::get,
        Router, Server,
    },
    std::collections::HashMap,
};

pub async fn run(conf: Net) {
    let app = Router::new().route("/:key", get(lookup));
    let addr = conf.socket_addr();
    println!("listening on http://{addr}");

    let service = app.into_make_service();
    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("error: {err}");
    }
}

async fn lookup(Path(path): Path<String>) -> Result<Redirect> {
    let map = HashMap::from([(String::from("lol"), String::from("https://lol.com"))]);
    let link = map.get(&path).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Redirect::temporary(link))
}
