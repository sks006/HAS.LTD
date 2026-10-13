use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequestDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequestDto {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,
}
