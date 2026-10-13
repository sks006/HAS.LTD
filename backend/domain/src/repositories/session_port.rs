// backend/domain/src/repositories/session_port.rs
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::errors::DomainError;
use crate::models::identifiers::{SessionId, UserId};
use crate::models::session::Session;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: Session) -> Result<Session, DomainError>;
    async fn find_by_id(&self, session_id: SessionId) -> Result<Option<Session>, DomainError>;
    async fn list_active_by_user_id(&self, user_id: UserId) -> Result<Vec<Session>, DomainError>;
    async fn touch_last_seen(&self, session_id: SessionId, at: DateTime<Utc>) -> Result<(), DomainError>;
    async fn revoke(&self, session_id: SessionId, at: DateTime<Utc>) -> Result<(), DomainError>;
    async fn revoke_all_for_user(&self, user_id: UserId, at: DateTime<Utc>) -> Result<u64, DomainError>;
}
