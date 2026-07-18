use axum::{ routing::{ get, post }, Router };

use crate::{ handlers::{ catalog::list_products, checkout::checkout }, state::AppState };

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { "ok" })
        )
        .route("/products", get(list_products))
        .route("/checkout", post(checkout))
        .with_state(state)
}
