// backend/api/src/routes.rs

use axum::{
    routing::{delete, get, post, put},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};

use crate::{
    handlers::{
        admin::{adjust_inventory, get_system_metrics},
        auth::{
            login, me, password_change, password_forgot, password_reset, register,
            register_resend_otp, register_verify_otp,
        },
        cart::sync_cart,
        catalog::{create_product, delete_product, get_product, list_products, update_product},
        checkout::checkout,
        order::{delete_order, get_order, list_orders, update_order_status},
        session::{delete_session, list_sessions, logout, logout_all, refresh},
        webhooks::handle_payment_webhook,
        mfa::{login_recovery, login_totp, mfa_disable, mfa_enroll, mfa_verify},
    },
    metrics::METRICS,
    state::AppState,
};

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn metrics_endpoint() -> impl axum::response::IntoResponse {
    let body = METRICS.render_prometheus();
    (
        [(axum::http::header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

pub fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Public observability
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint))

        // Product catalog (public reads, admin writes)
        .route("/products", get(list_products).post(create_product))
        .route("/products/{id}", get(get_product).put(update_product).delete(delete_product))

        // Checkout & cart (customer protected)
        .route("/cart/sync", post(sync_cart))
        .route("/checkout", post(checkout))

        // Orders (moderator/admin reads & updates, customer detail, admin delete)
        .route("/orders", get(list_orders))
        .route("/orders/{id}", get(get_order).delete(delete_order))
        .route("/orders/{id}/status", put(update_order_status))

        // Admin routes (admin protected)
        .route("/admin/inventory", post(adjust_inventory))
        .route("/admin/metrics", get(get_system_metrics))

        // Webhooks (HMAC verification)
        .route("/webhooks/payment", post(handle_payment_webhook))

        // Phase 4 Auth routes
        .route("/auth/login", post(login))
        .route("/auth/me", get(me))

        // Phase 5 Session & Token Management routes
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/logout-all", post(logout_all))
        .route("/auth/sessions", get(list_sessions))
        .route("/auth/sessions/{id}", delete(delete_session))

        // Phase 6A Email Flow routes
        .route("/auth/register", post(register))
        .route("/auth/register/verify-otp", post(register_verify_otp))
        .route("/auth/register/resend-otp", post(register_resend_otp))
        .route("/auth/password/forgot", post(password_forgot))
        .route("/auth/password/reset", post(password_reset))
        .route("/auth/password/change", post(password_change))

        // Phase 6B MFA (TOTP) routes
        .route("/auth/mfa/totp/enroll", post(mfa_enroll))
        .route("/auth/mfa/totp/verify", post(mfa_verify))
        .route("/auth/mfa/totp/disable", post(mfa_disable))
        .route("/auth/login/totp", post(login_totp))
        .route("/auth/login/recovery", post(login_recovery))

        .layer(cors)
        .with_state(state)
}
