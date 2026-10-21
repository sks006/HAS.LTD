// backend/api/src/dtos/auth_dto.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub role: String,
    pub status: String,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub mfa_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserDto>,
}
