// backend/domain/src/models/auth.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::user::UserRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (User ID)
    pub sub: Uuid, 
    /// Role for RBAC routing (Customer vs Admin)
    pub role: UserRole,
    /// JWT ID (Mandatory for revocation/blacklisting)
    pub jti: Uuid, 
    /// Issued At (Unix timestamp)
    pub iat: usize, 
    /// Expiration (Unix timestamp)
    pub exp: usize, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenPurpose {
    EmailVerification,
    PasswordReset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String, 
    pub purpose: TokenPurpose, // MANDATORY: Cryptographic intent isolation
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}