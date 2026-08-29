use axum::{ routing::{ get, post, put }, Router };

use crate::{
    handlers::{
        catalog::{list_products, get_product},
        checkout::checkout,
        order::{get_order, update_order_status},
        auth::{login, register, logout},
        cart::sync_cart,
        admin::{adjust_inventory, get_system_metrics},
        webhooks::handle_payment_webhook,
    },
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { "ok" })
        )
        .route("/products", get(list_products))
        .route("/products/{id}", get(get_product))
        .route("/checkout", post(checkout))
        .route("/orders/{id}", get(get_order))
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
        
        .with_state(state)
}

