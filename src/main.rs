use askama::Template;
use axum::{response::Html, routing::get, Router};
use tower_http::services::ServeDir;

#[derive(Template)]
#[template(path = "index.html")]
struct HomePage;

async fn home() -> Html<String> {
    Html(HomePage.render().unwrap())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    println!("Listening on http://localhost:3002");
    axum::serve(listener, app).await.unwrap();
}
