use axum::{Json, response::IntoResponse};
use crate::dtos::{LoginRequestDto, RegisterRequestDto};
use crate::errors::ApiError;

pub async fn login(Json(_payload): Json<LoginRequestDto>) -> Result<impl IntoResponse, ApiError> {
    // In production, verify credentials using AuthService and generate JWT token
    Ok(Json(serde_json::json!({
        "token": "mock-jwt-token-string",
        "role": "CUSTOMER"
    })))
}

pub async fn register(Json(_payload): Json<RegisterRequestDto>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "User registered successfully"
    })))
}

pub async fn logout() -> Result<impl IntoResponse, ApiError> {
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Logged out and token revoked"
    })))
}
