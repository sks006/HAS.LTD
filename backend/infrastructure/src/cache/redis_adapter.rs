// backend/infrastructure/src/cache/redis_adapter.rs

use async_trait::async_trait;
use redis::Client;
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
    async fn set_idempotency_key(&self, _key: &str, _value: &str, _ttl_seconds: u64) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn get_idempotency_key(&self, _key: &str) -> Result<Option<String>, DomainError> {
        unimplemented!()
    }

    async fn blacklist_jwt(&self, _jti: Uuid, _ttl_seconds: u64) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn is_jwt_blacklisted(&self, _jti: Uuid) -> Result<bool, DomainError> {
        unimplemented!()
    }
}
