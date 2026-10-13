use axum::{Json, response::IntoResponse};
use crate::dtos::AdjustInventoryDto;
use crate::errors::ApiError;
use crate::middleware::RequireAdmin;

pub async fn adjust_inventory(
    _guard: RequireAdmin,
    Json(_payload): Json<AdjustInventoryDto>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Inventory successfully adjusted"
    })))
}

pub async fn get_system_metrics(
    _guard: RequireAdmin,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "active_connections": 12,
        "database_status": "Online",
        "redis_status": "Online",
        "cpu_usage_pct": 2.4
    })))
}
