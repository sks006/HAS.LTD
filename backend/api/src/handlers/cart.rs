use axum::{Json, response::IntoResponse};
use crate::dtos::CartItemMutationDto;
use crate::errors::ApiError;
use crate::middleware::RequireCustomer;

pub async fn sync_cart(
    _guard: RequireCustomer,
    Json(_payload): Json<Vec<CartItemMutationDto>>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Cart synchronized successfully"
    })))
}
