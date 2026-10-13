// backend/domain/src/models/credentials.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::identifiers::{CredentialId, MfaCredentialId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "credential_kind_enum", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CredentialKind {
    Password,
    Social,
    Webauthn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Credentials {
    pub id: CredentialId,
    pub user_id: UserId,
    pub kind: CredentialKind,
    pub provider: Option<String>,
    pub provider_subject: Option<String>,
    pub secret_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct MfaCredential {
    pub id: MfaCredentialId,
    pub user_id: UserId,
    pub kind: String, // 'totp'
    pub secret_encrypted: Vec<u8>,
    pub recovery_codes_hash: Vec<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
