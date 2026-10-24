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
    #[serde(default)]
    pub mfa_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_enrollment_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDto {
    pub id: Uuid,
    pub current: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_fingerprint: Option<String>,
    pub user_agent: String,
    pub ip_address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutAllResponse {
    pub success: bool,
    pub sessions_revoked: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    
    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    pub password: String,

    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(equal = 6, message = "OTP must be exactly 6 digits"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOtpResponse {
    pub message: String,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResendOtpRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResendOtpResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgotPasswordResponse {
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(equal = 6, message = "OTP must be exactly 6 digits"))]
    pub code: String,

    #[validate(length(min = 8, max = 128, message = "Password must be between 8 and 128 characters"))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetPasswordResponse {
    pub message: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "Current password cannot be empty"))]
    pub current_password: String,

    #[validate(length(min = 8, max = 128, message = "New password must be between 8 and 128 characters"))]
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordResponse {
    pub message: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EnrollMfaRequest {
    #[validate(length(min = 1, message = "Current password cannot be empty"))]
    pub current_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollMfaResponse {
    pub otpauth_url: String,
    pub secret: String,
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerifyEnrollmentRequest {
    #[validate(length(equal = 6, message = "Code must be exactly 6 digits"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DisableMfaRequest {
    #[validate(length(min = 1, message = "Current password cannot be empty"))]
    pub current_password: String,
    #[validate(length(min = 6, max = 10, message = "Code must be a 6-digit TOTP code or a 10-character recovery code"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginTotpRequest {
    #[validate(length(min = 1, message = "mfa_token cannot be empty"))]
    pub mfa_token: String,
    #[validate(length(equal = 6, message = "Code must be exactly 6 digits"))]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRecoveryRequest {
    #[validate(length(min = 1, message = "mfa_token cannot be empty"))]
    pub mfa_token: String,
    #[validate(length(equal = 10, message = "Recovery code must be exactly 10 characters"))]
    pub recovery_code: String,
}
