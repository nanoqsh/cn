use {
    crate::config::Config,
    axum::{response::Html, routing::get, Router, Server},
};

pub async fn run(conf: Config) {
    let app = Router::new().route("/", get(handler));
    let addr = conf.socket_addr();
    println!("listening on http://{addr}");

    let service = app.into_make_service();
    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("error: {err}");
    }
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
