use axum::{routing::get, Extension, Router};
use std::sync::Arc;

mod handler;
mod message;
mod state;

#[tokio::main]
async fn main() {
    let proxy_url = std::env::var("PROXY_URL").unwrap_or("0.0.0.0:4000".to_string());

    let destination_url =
        std::env::var("DESTINATION_URL").unwrap_or("ws://0.0.0.0:3000".to_string());

    let state = Arc::new(state::State { destination_url });

    let app = Router::new()
        .route("/proxy", get(handler::handler))
        .layer(Extension(state));

    axum::Server::bind(&proxy_url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
