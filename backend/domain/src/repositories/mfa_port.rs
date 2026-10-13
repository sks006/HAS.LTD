// backend/domain/src/repositories/mfa_port.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::errors::DomainError;
use crate::models::identifiers::UserId;
use crate::models::credentials::MfaCredential;

#[async_trait]
pub trait MfaRepository: Send + Sync {
    async fn upsert(&self, mfa: MfaCredential) -> Result<MfaCredential, DomainError>;
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<MfaCredential>, DomainError>;
    async fn confirm(&self, user_id: UserId, at: DateTime<Utc>) -> Result<(), DomainError>;
    async fn delete_by_user_id(&self, user_id: UserId) -> Result<(), DomainError>;
}
