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

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://localhost:{port}");
    axum::serve(listener, app).await.unwrap();
}
