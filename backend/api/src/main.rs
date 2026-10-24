// backend/api/src/main.rs

use api::{routes::build_router, state::AppState};
use infrastructure::tracing::init_tracing;

#[tokio::main]
async fn main() {
    let log_format = std::env::var("RUST_LOG_FORMAT").unwrap_or_else(|_| "pretty".to_string());
    init_tracing(&log_format);

    let state = AppState::new().await.expect("Failed to initialize AppState");

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
