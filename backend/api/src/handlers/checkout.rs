use axum::{ extract::State, http::StatusCode, Json };
use serde::Serialize;

use crate::{ extractors::{ AuthExtractor, IdempotencyExtractor }, state::AppState };

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub ok: bool,
    pub message: String,
}

pub async fn checkout(
    State(_state): State<AppState>,
    AuthExtractor(auth): AuthExtractor,
    IdempotencyExtractor(idempotency_key): IdempotencyExtractor
) -> Result<Json<CheckoutResponse>, StatusCode> {
    let _ = (auth, idempotency_key);

    Ok(
        Json(CheckoutResponse {
            ok: true,
            message: "checkout accepted".to_string(),
        })
    )
}
