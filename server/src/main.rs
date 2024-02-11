use axum::{routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    let serve_dir = ServeDir::new("../client/dist")
        .not_found_service(ServeFile::new("../client/dist/index.html"));

    let app = Router::new()
        .route("/api", get(|| async { "Hello, World!" }))
        .fallback_service(serve_dir);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
