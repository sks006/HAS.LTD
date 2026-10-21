// backend/domain/src/models/session.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::identifiers::{RefreshTokenId, SessionId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub device_fingerprint: Option<String>,
    pub user_agent: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: RefreshTokenId,
    pub session_id: SessionId,
    pub family_id: Uuid,
    pub token_hash: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub replaced_by_id: Option<RefreshTokenId>,
}
