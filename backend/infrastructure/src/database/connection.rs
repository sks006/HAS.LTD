// backend/infrastructure/src/database/connection.rs

use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use sqlx::PgPool;
use std::time::Duration;
use std::str::FromStr;
use domain::errors::DomainError;

pub struct DatabaseConnection;

impl DatabaseConnection {
    /// Creates and configures a PostgreSQL connection pool.
    pub async fn create_pool(database_url: &str) -> Result<PgPool, DomainError> {
        let connection_options = PgConnectOptions::from_str(database_url)
            .map_err(|e| DomainError::DatabaseError(format!("Invalid database connection URL: {}", e)))?;

        PgPoolOptions::new()
            .max_connections(20)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600))
            .connect_with(connection_options)
            .await
            .map_err(|e| DomainError::DatabaseError(format!("Failed to connect to Postgres: {}", e)))
    }

    /// Performs a TCP / database query health check on the PgPool.
    pub async fn check_health(pool: &PgPool) -> Result<(), DomainError> {
        sqlx::query("SELECT 1")
            .execute(pool)
            .await
            .map_err(|e| DomainError::DatabaseError(format!("Database health check failed: {}", e)))?;
        Ok(())
    }
}
