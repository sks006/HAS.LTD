use axum::{Json, response::IntoResponse};
use crate::dtos::CartItemMutationDto;
use crate::errors::ApiError;

pub async fn sync_cart(Json(_payload): Json<Vec<CartItemMutationDto>>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Cart synchronized successfully"
    })))
}
