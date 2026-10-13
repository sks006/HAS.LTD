// backend/domain/src/repositories/cache_port.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::errors::DomainError;

#[async_trait]
pub trait CachePort: Send + Sync {
    async fn set_idempotency_key(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), DomainError>;
    async fn get_idempotency_key(&self, key: &str) -> Result<Option<String>, DomainError>;
    async fn blacklist_jwt(&self, jti: Uuid, ttl_seconds: u64) -> Result<(), DomainError>;
    async fn is_jwt_blacklisted(&self, jti: Uuid) -> Result<bool, DomainError>;
}
