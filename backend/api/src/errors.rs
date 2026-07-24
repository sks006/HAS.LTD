// backend/api/src/errors.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use domain::errors::DomainError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::Forbidden(msg) => (StatusCode::FORBIDDEN, "forbidden", msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
        };

        let body = Json(json!({
            "error": error_type,
            "message": message
        }));

        (status, body).into_response()
    }
}

impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::InsufficientStock { variant_id, requested, available } => {
                ApiError::BadRequest(format!(
                    "Insufficient stock for variant {}: requested {}, available {}",
                    variant_id, requested, available
                ))
            }
            DomainError::ConcurrentStateModification => {
                ApiError::Conflict("Concurrent state modification. Please retry.".to_string())
            }
            DomainError::IdempotencyConflict => {
                ApiError::Conflict("Idempotent request conflict".to_string())
            }
            DomainError::VariantNotFound => {
                ApiError::NotFound("Product variant not found".to_string())
            }
            DomainError::ProductNotFound => {
                ApiError::NotFound("Product not found".to_string())
            }
            DomainError::SnapshotNotFound(id) => {
                ApiError::NotFound(format!("Stock snapshot not found for variant {}", id))
            }
            DomainError::OrderNotFound => {
                ApiError::NotFound("Order not found".to_string())
            }
            DomainError::UserNotFound => {
                ApiError::NotFound("User not found".to_string())
            }
            DomainError::UserAlreadyExists => {
                ApiError::Conflict("User already exists".to_string())
            }
            DomainError::Unauthorized => {
                ApiError::Unauthorized("Unauthorized access".to_string())
            }
            DomainError::InvalidStateTransition => {
                ApiError::BadRequest("Invalid state transition".to_string())
            }
            DomainError::CurrencyMismatch { variant_id, variant_currency, order_currency } => {
                ApiError::BadRequest(format!(
                    "Currency mismatch for variant {}: variant has {}, order has {}",
                    variant_id, variant_currency, order_currency
                ))
            }
            DomainError::DatabaseError(msg) | DomainError::Database(msg) => {
                ApiError::Internal(format!("Database error: {}", msg))
            }
            DomainError::CacheError(msg) => {
                ApiError::Internal(format!("Cache error: {}", msg))
            }
            DomainError::Serialization(msg) => {
                ApiError::Internal(format!("Serialization error: {}", msg))
            }
            DomainError::IdempotencyCollision(key) => {
                ApiError::Conflict(format!("Idempotency key collision for key: {}", key))
            }
            DomainError::ConcurrencyConflict { message } => {
                ApiError::Conflict(message)
            }
            DomainError::InvalidEnumValue(msg) => {
                ApiError::BadRequest(format!("Invalid enum value: {}", msg))
            }
        }
    }
}
