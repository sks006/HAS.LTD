mod dtos;
mod errors;
mod extractors;
mod handlers;
mod routes;
mod state;

use crate::{ routes::build_router, state::AppState };

#[tokio::main]
async fn main() {
    let state = AppState::new().await.expect("Failed to initialize AppState");

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
