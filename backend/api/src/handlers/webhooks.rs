use axum::{Json, response::IntoResponse};
use crate::errors::ApiError;

pub async fn handle_payment_webhook(payload: String) -> Result<impl IntoResponse, ApiError> {
    // In production, verifies payment gateway HMAC-SHA256 signature
    tracing::info!("Received webhook payload size: {}", payload.len());
    Ok(Json(serde_json::json!({
        "status": "received"
    })))
}
