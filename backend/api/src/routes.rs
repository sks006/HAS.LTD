use axum::{ routing::{ get, post, put }, Router };
use tower_http::cors::{Any, CorsLayer};

use crate::{
    handlers::{
        catalog::{list_products, get_product, create_product, update_product, delete_product},
        checkout::checkout,
        order::{list_orders, get_order, update_order_status, delete_order},
        auth::{login, register, logout},
        cart::sync_cart,
        admin::{adjust_inventory, get_system_metrics},
        webhooks::handle_payment_webhook,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route(
            "/health",
            get(|| async { "ok" })
        )
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}", get(get_product).put(update_product).delete(delete_product))
        .route("/checkout", post(checkout))
        .route("/orders", get(list_orders))
        .route("/orders/{id}", get(get_order).delete(delete_order))
        .route("/orders/{id}/status", put(update_order_status))
        
        // Auth routes
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        
        // Cart routes
        .route("/cart/sync", post(sync_cart))
        
        // Admin routes
        .route("/admin/inventory", post(adjust_inventory))
        .route("/admin/metrics", get(get_system_metrics))
        
        // Webhooks
        .route("/webhooks/payment", post(handle_payment_webhook))
        
        .layer(cors)
        .with_state(state)
}
