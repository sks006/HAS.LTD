use axum::{
    async_trait,
    extract::{ FromRequestParts, TypedHeader },
    http::request::Parts,
    response::{ IntoResponse, Response },
    Json,
};
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthClaim {
    pub sub: String,
}

pub struct AuthExtractor(pub AuthClaim);

#[async_trait]
impl<S> FromRequestParts<S> for AuthExtractor where S: Send + Sync {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get("x-user-id");
        match auth_header {
            Some(value) =>
                Ok(
                    Self(AuthClaim {
                        sub: value.to_str().unwrap_or_default().to_string(),
                    })
                ),
            None =>
                Err((axum::http::StatusCode::UNAUTHORIZED, "missing x-user-id").into_response()),
        }
    }
}

pub struct IdempotencyExtractor(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for IdempotencyExtractor where S: Send + Sync {
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let idempotency_key = parts.headers.get("idempotency-key");
        match idempotency_key {
            Some(value) => Ok(Self(value.to_str().unwrap_or_default().to_string())),
            None =>
                Err(
                    (axum::http::StatusCode::BAD_REQUEST, "missing idempotency-key").into_response()
                ),
        }
    }
}
