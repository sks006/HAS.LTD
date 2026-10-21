// backend/api/src/state.rs

use std::sync::Arc;
use std::time::Duration;
use dotenvy::dotenv;
use sqlx::PgPool;
use tokio::sync::Semaphore;
use crypto::jwt::JwtKeys;
use crate::config::AppConfig;
use crate::middleware::argon2_gate::Argon2Gate;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis: Option<redis::aio::ConnectionManager>,
    pub jwt_keys: Arc<JwtKeys>,
    pub argon2_semaphore: Arc<Semaphore>,
    pub config: Arc<AppConfig>,
}

impl AppState {
    /// Constructs AppState by loading configuration, database connection pool,
    /// Redis connection manager, RSA JWT keypairs, and Argon2 concurrency semaphore.
    ///
    /// Per Phase 3 §1.h: If Redis is unavailable at boot, the application logs a warning
    /// and boots in degraded mode (fail-open for rate limiting and cache fallbacks).
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();

        let config = Arc::new(AppConfig::from_env());

        let db_pool = infrastructure::database::connection::DatabaseConnection::create_pool(&config.database_url)
            .await
            .map_err(|e| format!("Database connection error: {:?}", e))?;

        let redis = match redis::Client::open(config.redis_url.as_str()) {
            Ok(client) => {
                match tokio::time::timeout(Duration::from_secs(3), redis::aio::ConnectionManager::new(client)).await {
                    Ok(Ok(manager)) => Some(manager),
                    Ok(Err(e)) => {
                        tracing::warn!("Redis connection failed at boot: {}. Operating in degraded mode.", e);
                        None
                    }
                    Err(_) => {
                        tracing::warn!("Redis connection timed out at boot. Operating in degraded mode.");
                        None
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Invalid Redis URL '{}': {}. Operating in degraded mode.", config.redis_url, e);
                None
            }
        };

        // Locate jwt directory, trying relative path or searching up to repo root
        let jwt_dir = if config.crypto.jwt_dir.exists() {
            config.crypto.jwt_dir.clone()
        } else {
            let alt_path = std::path::PathBuf::from("../../.secrets/jwt");
            if alt_path.exists() {
                alt_path
            } else {
                config.crypto.jwt_dir.clone()
            }
        };

        let jwt_keys = Arc::new(
            JwtKeys::load_from_dir(&jwt_dir)
                .map_err(|e| format!("Failed to load JWT keys from {}: {:?}", jwt_dir.display(), e))?,
        );

        let argon2_semaphore = Arc::new(Semaphore::new(config.argon2_permits));

        Ok(Self {
            db_pool,
            redis,
            jwt_keys,
            argon2_semaphore,
            config,
        })
    }

    /// Helper to instantiate an Argon2Gate concurrency wrapper with configured semaphore and timeout.
    pub fn argon2_gate(&self) -> Argon2Gate {
        Argon2Gate::from_semaphore(
            self.argon2_semaphore.clone(),
            Duration::from_millis(self.config.argon2_timeout_ms),
        )
    }

    /// Test constructor allowing direct injection of state dependencies.
    #[allow(dead_code)]
    pub fn from_parts(
        db_pool: PgPool,
        redis: Option<redis::aio::ConnectionManager>,
        jwt_keys: Arc<JwtKeys>,
        argon2_semaphore: Arc<Semaphore>,
        config: Arc<AppConfig>,
    ) -> Self {
        Self {
            db_pool,
            redis,
            jwt_keys,
            argon2_semaphore,
            config,
        }
    }
}