// backend/api/src/config.rs

use std::env;
use crypto::config::CryptoConfig;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub argon2_permits: usize,
    pub argon2_timeout_ms: u64,
    pub crypto: CryptoConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:test123@127.0.0.1:5432/has".to_string());
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let argon2_permits = env::var("ARGON2_SEMAPHORE_PERMITS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);
        let argon2_timeout_ms = env::var("ARGON2_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);

        let crypto = CryptoConfig::load().unwrap_or_default();

        Self {
            database_url,
            redis_url,
            argon2_permits,
            argon2_timeout_ms,
            crypto,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
