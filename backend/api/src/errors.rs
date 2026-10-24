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
    BadRequestWithCode { code: String, message: String },
    Unauthorized(String),
    UnauthorizedWithCode { code: String, message: String },
    ForbiddenWithCode { code: String, message: String },
    NotFound(String),
    NotFoundWithCode { code: String, message: String },
    Conflict(String),
    ConflictWithCode { code: String, message: String },
    RefreshInProgress { retry_after: u64 },
    RateLimited { retry_after: u64 },
    NotImplemented { code: String, message: String },
    ServiceUnavailable(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, retry_after) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request".to_string(), msg, None),
            ApiError::BadRequestWithCode { code, message } => (StatusCode::BAD_REQUEST, code, message, None),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized".to_string(), msg, None),
            ApiError::UnauthorizedWithCode { code, message } => (StatusCode::UNAUTHORIZED, code, message, None),
            ApiError::ForbiddenWithCode { code, message } => (StatusCode::FORBIDDEN, code, message, None),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found".to_string(), msg, None),
            ApiError::NotFoundWithCode { code, message } => (StatusCode::NOT_FOUND, code, message, None),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, "conflict".to_string(), msg, None),
            ApiError::ConflictWithCode { code, message } => (StatusCode::CONFLICT, code, message, None),
            ApiError::RefreshInProgress { retry_after } => (
                StatusCode::CONFLICT,
                "refresh_in_progress".to_string(),
                "A token refresh is currently being processed for this session.".to_string(),
                Some(retry_after),
            ),
            ApiError::RateLimited { retry_after } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited".to_string(),
                "Too many requests. Please try again later.".to_string(),
                Some(retry_after),
            ),
            ApiError::NotImplemented { code, message } => (StatusCode::NOT_IMPLEMENTED, code, message, None),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, "service_unavailable".to_string(), msg, None),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error".to_string(), msg, None),
        };

        let body = Json(json!({
            "error": {
                "code": code,
                "message": message,
            }
        }));

        let mut response = (status, body).into_response();
        if let Some(secs) = retry_after {
            if let Ok(val) = axum::http::HeaderValue::from_str(&secs.to_string()) {
                response.headers_mut().insert(axum::http::header::RETRY_AFTER, val);
            }
        }
        response
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
