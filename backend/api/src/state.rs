// backend/api/src/state.rs

use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use dotenvy::dotenv; // optional – load .env file

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: Arc<String>,
}

impl AppState {
    /// Create a new AppState by reading DATABASE_URL and JWT_SECRET from the environment.
    /// Panics if either variable is missing – we want to fail fast at boot.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok(); // load .env if present (optional)

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");

        let db_pool = infrastructure::database::connection::DatabaseConnection::create_pool(&database_url)
            .await
            .map_err(|e| format!("Database connection error: {:?}", e))?;
        
        Ok(Self {
            db_pool,
            jwt_secret: Arc::new(jwt_secret),
        })
    }

    // If you need to inject a pool for testing, you can use a separate constructor.
    #[cfg(test)]
    pub fn from_pool_and_secret(db_pool: PgPool, jwt_secret: String) -> Self {
        Self {
            db_pool,
            jwt_secret: Arc::new(jwt_secret),
        }
    }
}