// backend/crypto/src/errors.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("password hashing failed")]
    PasswordHashFailed,

    #[error("password verification failed")]
    PasswordVerifyFailed,

    #[error("invalid password hash format: {0}")]
    InvalidPasswordHash(String),

    #[error("token generation failed: {0}")]
    TokenGenerationFailed(String),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("JWT header missing key ID (kid)")]
    MissingKeyId,

    #[error("JWT key ID (kid) unknown: {0}")]
    UnknownKeyId(String),

    #[error("unsupported JWT algorithm (only RS256 is accepted)")]
    UnsupportedAlgorithm,

    #[error("invalid token claims: {0}")]
    InvalidClaims(String),

    #[error("TOTP error: {0}")]
    TotpError(String),

    #[error("TOTP AEAD encryption failed")]
    AeadEncryptionFailed,

    #[error("TOTP AEAD decryption failed: data corrupted or incorrect key")]
    AeadDecryptionFailed,

    #[error("invalid key length: expected {expected} bytes, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("crypto configuration error: {0}")]
    ConfigError(String),

    #[error("task execution failed: {0}")]
    TaskJoinError(String),

    #[error("I/O error during crypto operation: {0}")]
    IoError(#[from] std::io::Error),
}
