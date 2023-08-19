use {
    axum::{response::Html, routing::get, Router, Server},
    std::net::SocketAddr,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on http://{addr}");

    let service = app.into_make_service();
    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("error: {err}");
    };
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
