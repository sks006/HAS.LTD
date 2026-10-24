// backend/api/src/config.rs

use std::env;
use crypto::config::CryptoConfig;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub argon2_permits: usize,
    pub argon2_timeout_ms: u64,
    pub resend_api_key: Option<String>,
    pub email_from: String,
    pub crypto: CryptoConfig,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    Missing(&'static str),
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Missing("DATABASE_URL"))?;
        let redis_url = std::env::var("REDIS_URL")
            .map_err(|_| ConfigError::Missing("REDIS_URL"))?;
        let argon2_permits = env::var("ARGON2_SEMAPHORE_PERMITS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);
        let argon2_timeout_ms = env::var("ARGON2_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2000);
        let resend_api_key = env::var("RESEND_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let email_from = env::var("EMAIL_FROM")
            .unwrap_or_else(|_| "HAS.LTD <noreply@has.ltd>".to_string());

        let crypto = CryptoConfig::load().unwrap_or_default();

        Ok(Self {
            database_url,
            redis_url,
            argon2_permits,
            argon2_timeout_ms,
            resend_api_key,
            email_from,
            crypto,
        })
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::from_env().expect("Required environment variables must be set")
    }
}
