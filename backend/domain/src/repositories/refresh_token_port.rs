// backend/domain/src/repositories/refresh_token_port.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::errors::DomainError;
use crate::models::identifiers::{RefreshTokenId, SessionId};
use crate::models::session::RefreshToken;

#[async_trait]
pub trait RefreshTokenRepository: Send + Sync {
    async fn create(&self, token: RefreshToken) -> Result<RefreshToken, DomainError>;
    async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<RefreshToken>, DomainError>;
    async fn revoke(&self, token_id: RefreshTokenId, replaced_by: Option<RefreshTokenId>, at: DateTime<Utc>) -> Result<(), DomainError>;
    async fn revoke_family(&self, family_id: Uuid, at: DateTime<Utc>) -> Result<u64, DomainError>;
    async fn revoke_by_session_id(&self, session_id: SessionId, at: DateTime<Utc>) -> Result<u64, DomainError>;
}
