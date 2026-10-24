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
    pub db_query_count: Arc<std::sync::atomic::AtomicUsize>,
    pub email_adapter: Arc<dyn domain::ports::email_port::EmailPort>,
    pub email_pepper: Arc<[u8; 32]>,
    pub in_memory_email: Option<Arc<infrastructure::email::InMemoryEmailAdapter>>,
    pub totp_key: Arc<[u8; 32]>,
}

impl AppState {
    /// Constructs AppState by loading configuration, database connection pool,
    /// Redis connection manager, RSA JWT keypairs, and Argon2 concurrency semaphore.
    ///
    /// Per Phase 3 §1.h: If Redis is unavailable at boot, the application logs a warning
    /// and boots in degraded mode (fail-open for rate limiting and cache fallbacks).
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();

        let config = Arc::new(AppConfig::from_env()?);

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
        let db_query_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // Email adapter selection per Pre-seeded Decision 1 & 5:
        // if RESEND_API_KEY is present -> ResendEmailAdapter, else -> InMemoryEmailAdapter
        let (email_adapter, in_memory_email): (
            Arc<dyn domain::ports::email_port::EmailPort>,
            Option<Arc<infrastructure::email::InMemoryEmailAdapter>>,
        ) = match &config.resend_api_key {
            Some(key) => {
                tracing::info!("Initializing ResendEmailAdapter with configured API key");
                (
                    Arc::new(infrastructure::email::ResendEmailAdapter::new(
                        key.clone(),
                        config.email_from.clone(),
                    )),
                    None,
                )
            }
            None => {
                tracing::info!("RESEND_API_KEY unset. Initializing InMemoryEmailAdapter as default");
                let mem = Arc::new(infrastructure::email::InMemoryEmailAdapter::new());
                (mem.clone(), Some(mem))
            }
        };

        let email_pepper_bytes = if config.crypto.email_pepper != [0u8; 32] {
            config.crypto.email_pepper
        } else {
            let pepper_path = if config.crypto.email_pepper_path.exists() {
                config.crypto.email_pepper_path.clone()
            } else {
                let alt = std::path::PathBuf::from("../../.secrets/email.pepper");
                if alt.exists() {
                    alt
                } else {
                    config.crypto.email_pepper_path.clone()
                }
            };
            let bytes = std::fs::read(&pepper_path)
                .map_err(|e| format!("Failed to read email pepper from {}: {:?}", pepper_path.display(), e))?;
            if bytes.len() != 32 {
                return Err(format!("email.pepper must be exactly 32 bytes, found {}", bytes.len()).into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            arr
        };

        let email_pepper = Arc::new(email_pepper_bytes);

        let totp_key_bytes = {
            let key_path = if config.crypto.totp_key_path.exists() {
                config.crypto.totp_key_path.clone()
            } else {
                let alt = std::path::PathBuf::from("../../.secrets/totp.key");
                if alt.exists() {
                    alt
                } else {
                    config.crypto.totp_key_path.clone()
                }
            };
            let bytes = std::fs::read(&key_path)
                .map_err(|e| format!("Failed to read totp key from {}: {:?}", key_path.display(), e))?;
            if bytes.len() != 32 {
                return Err(format!("totp.key must be exactly 32 bytes, found {}", bytes.len()).into());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            arr
        };
        let totp_key = Arc::new(totp_key_bytes);

        Ok(Self {
            db_pool,
            redis,
            jwt_keys,
            argon2_semaphore,
            config,
            db_query_count,
            email_adapter,
            email_pepper,
            in_memory_email,
            totp_key,
        })
    }

    /// Counts an executed database query: increments the atomic test counter AND emits the auth_db_query_total metric.
    pub fn count_db_query(&self) {
        self.db_query_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        crate::metrics::METRICS.counter_inc("auth_db_query_total");
    }

    /// Records an executed database query for instrumentation and cache verification (aliases count_db_query).
    pub fn record_db_query(&self) {
        self.count_db_query();
    }

    /// Returns the number of database queries executed since state creation.
    pub fn get_db_query_count(&self) -> usize {
        self.db_query_count.load(std::sync::atomic::Ordering::SeqCst)
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
        let (email_adapter, in_memory_email): (
            Arc<dyn domain::ports::email_port::EmailPort>,
            Option<Arc<infrastructure::email::InMemoryEmailAdapter>>,
        ) = match &config.resend_api_key {
            Some(key) => (
                Arc::new(infrastructure::email::ResendEmailAdapter::new(
                    key.clone(),
                    config.email_from.clone(),
                )),
                None,
            ),
            None => {
                let mem = Arc::new(infrastructure::email::InMemoryEmailAdapter::new());
                (mem.clone(), Some(mem))
            }
        };

        let email_pepper_bytes = if config.crypto.email_pepper != [0u8; 32] {
            config.crypto.email_pepper
        } else {
            let pepper_path = if config.crypto.email_pepper_path.exists() {
                config.crypto.email_pepper_path.clone()
            } else {
                let alt = std::path::PathBuf::from("../../.secrets/email.pepper");
                if alt.exists() {
                    alt
                } else {
                    config.crypto.email_pepper_path.clone()
                }
            };
            let bytes = std::fs::read(&pepper_path).unwrap_or_else(|_| vec![0u8; 32]);
            let mut arr = [0u8; 32];
            if bytes.len() == 32 {
                arr.copy_from_slice(&bytes);
            }
            arr
        };

        let totp_key_bytes = {
            let key_path = if config.crypto.totp_key_path.exists() {
                config.crypto.totp_key_path.clone()
            } else {
                let alt = std::path::PathBuf::from("../../.secrets/totp.key");
                if alt.exists() {
                    alt
                } else {
                    config.crypto.totp_key_path.clone()
                }
            };
            let bytes = std::fs::read(&key_path).unwrap_or_else(|_| vec![0u8; 32]);
            let mut arr = [0u8; 32];
            if bytes.len() == 32 {
                arr.copy_from_slice(&bytes);
            }
            arr
        };

        Self {
            db_pool,
            redis,
            jwt_keys,
            argon2_semaphore,
            config: config.clone(),
            db_query_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            email_adapter,
            email_pepper: Arc::new(email_pepper_bytes),
            in_memory_email,
            totp_key: Arc::new(totp_key_bytes),
        }
    }
}