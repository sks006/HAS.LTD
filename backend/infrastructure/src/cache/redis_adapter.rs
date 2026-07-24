// backend/infrastructure/src/cache/redis_adapter.rs

use async_trait::async_trait;
use redis::{Client, AsyncCommands};
use uuid::Uuid;
use domain::errors::DomainError;
use domain::repositories::CachePort;

pub struct RedisCacheAdapter {
    pub client: Client,
}

impl RedisCacheAdapter {
    pub fn new(redis_url: &str) -> Result<Self, DomainError> {
        let client = Client::open(redis_url)
            .map_err(|e| DomainError::CacheError(format!("Failed to connect to Redis: {}", e)))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl CachePort for RedisCacheAdapter {
    async fn set_idempotency_key(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<(), DomainError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::CacheError(format!("Redis connection failed: {}", e)))?;
        conn.set_ex(key, value, ttl_seconds).await
            .map_err(|e| DomainError::CacheError(format!("Redis SETEX failed: {}", e)))
    }

    async fn get_idempotency_key(&self, key: &str) -> Result<Option<String>, DomainError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::CacheError(format!("Redis connection failed: {}", e)))?;
        conn.get(key).await
            .map_err(|e| DomainError::CacheError(format!("Redis GET failed: {}", e)))
    }

    async fn blacklist_jwt(&self, jti: Uuid, ttl_seconds: u64) -> Result<(), DomainError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::CacheError(format!("Redis connection failed: {}", e)))?;
        let key = format!("jwt:blacklist:{}", jti);
        conn.set_ex(key, "true", ttl_seconds).await
            .map_err(|e| DomainError::CacheError(format!("Redis SETEX for JWT blacklist failed: {}", e)))
    }

    async fn is_jwt_blacklisted(&self, jti: Uuid) -> Result<bool, DomainError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| DomainError::CacheError(format!("Redis connection failed: {}", e)))?;
        let key = format!("jwt:blacklist:{}", jti);
        let exists: bool = conn.exists(key).await
            .map_err(|e| DomainError::CacheError(format!("Redis EXISTS check failed: {}", e)))?;
        Ok(exists)
    }
}
