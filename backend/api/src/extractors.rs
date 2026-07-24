//! Request extractors for authentication and idempotency.
//! `RequireCustomer` validates JWT and enforces role‑based access control.

use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
    http::header::AUTHORIZATION,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use domain::models::auth::JwtClaims;
use domain::models::user::UserRole;
use crate::state::AppState;
use crate::errors::ApiError;

/// Extractor that ensures the request is authenticated as a Customer.
/// Returns the decoded JWT claims on success.
pub struct RequireCustomer(pub JwtClaims);

impl<S> FromRequestParts<S> for RequireCustomer
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Internal("Failed to extract application state".to_string()))?;

        let auth_header = parts.headers
            .get(AUTHORIZATION)
            .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| ApiError::Unauthorized("Invalid Authorization header encoding".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized("Invalid authorization scheme; expected Bearer".to_string()));
        }
        let token = &auth_str[7..];

        let secret = app_state.jwt_secret.as_str();
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        let token_data = decode::<JwtClaims>(
            token,
            &decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ApiError::Unauthorized("Invalid or expired JWT token".to_string()))?;

        if token_data.claims.role != UserRole::Customer {
            return Err(ApiError::Forbidden("This endpoint requires Customer role".to_string()));
        }

        Ok(RequireCustomer(token_data.claims))
    }
}

/// Extractor that decodes JWT claims for general authentication.
pub struct AuthExtractor(pub JwtClaims);

impl<S> FromRequestParts<S> for AuthExtractor
where
    S: Send + Sync,
    AppState: axum::extract::FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let State(app_state) = State::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Internal("Failed to extract application state".to_string()))?;

        let auth_header = parts.headers
            .get(AUTHORIZATION)
            .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| ApiError::Unauthorized("Invalid Authorization header encoding".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized("Invalid authorization scheme; expected Bearer".to_string()));
        }
        let token = &auth_str[7..];

        let secret = app_state.jwt_secret.as_str();
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        let token_data = decode::<JwtClaims>(
            token,
            &decoding_key,
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ApiError::Unauthorized("Invalid or expired JWT token".to_string()))?;

        Ok(AuthExtractor(token_data.claims))
    }
}

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