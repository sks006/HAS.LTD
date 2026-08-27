use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::errors::ApiError;

/// Extractor that retrieves and validates the idempotency key from headers.
pub struct IdempotencyExtractor(pub String);

impl<S> FromRequestParts<S> for IdempotencyExtractor
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = parts.headers
            .get("idempotency-key")
            .or_else(|| parts.headers.get("x-idempotency-key"))
            .ok_or_else(|| ApiError::BadRequest("Missing Idempotency-Key header".to_string()))?;

        let key_str = key.to_str()
            .map_err(|_| ApiError::BadRequest("Invalid Idempotency-Key encoding".to_string()))?;

        if key_str.trim().is_empty() {
            return Err(ApiError::BadRequest("Idempotency-Key cannot be empty".to_string()));
        }

        Ok(IdempotencyExtractor(key_str.to_string()))
    }
}
