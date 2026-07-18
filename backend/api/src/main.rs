mod extractors;
mod handlers;
mod routes;
mod state;

use std::sync::Arc;

use crate::{ routes::build_router, state::{ AppConfig, AppState } };

#[tokio::main]
async fn main() {
    let state = AppState {
        config: Arc::new(AppConfig::default()),
    };

    let app = build_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
