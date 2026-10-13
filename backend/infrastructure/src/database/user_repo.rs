// backend/infrastructure/src/database/user_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::auth::VerificationToken;
use domain::models::user::{User, UserIdentity};
use domain::repositories::UserRepository;

use domain::models::user::UserRole;
use domain::models::auth::TokenPurpose;

fn user_role_to_str(role: &UserRole) -> &'static str {
    role.as_str()
}

fn str_to_user_role(role_str: &str) -> Result<UserRole, DomainError> {
    UserRole::try_from(role_str)
        .map_err(|e| DomainError::InvalidEnumValue(e))
}

fn token_purpose_to_str(purpose: &TokenPurpose) -> &'static str {
    match purpose {
        TokenPurpose::EmailVerification => "EmailVerification",
        TokenPurpose::PasswordReset => "PasswordReset",
    }
}

fn str_to_token_purpose(purpose_str: &str) -> Result<TokenPurpose, DomainError> {
    match purpose_str {
        "EmailVerification" => Ok(TokenPurpose::EmailVerification),
        "PasswordReset" => Ok(TokenPurpose::PasswordReset),
        _ => Err(DomainError::InvalidEnumValue(format!("Unknown token purpose: {}", purpose_str))),
    }
}

pub struct PgUserRepository {
    pub pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(&self, user: User, password_hash: String) -> Result<User, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, email_normalized, name, role, status, auth_version, email_verified_at, created_at, updated_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.email_normalized)
        .bind(&user.name)
        .bind(user_role_to_str(&user.role))
        .bind(user.status)
        .bind(user.auth_version)
        .bind(user.email_verified_at)
        .bind(user.created_at)
        .bind(user.updated_at)
        .bind(&user.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return DomainError::UserAlreadyExists;
                }
            }
            DomainError::Database(e.to_string())
        })?;

        // If credentials table exists, store the password there
        let _ = sqlx::query(
            r#"
            INSERT INTO credentials (id, user_id, kind, provider, provider_subject, secret_hash, created_at, updated_at)
            VALUES ($1, $2, 'password', 'local', NULL, $3, $4, $5)
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(uuid::Uuid::now_v7())
        .bind(user.id.0)
        .bind(&password_hash)
        .bind(user.created_at)
        .bind(user.updated_at)
        .execute(&self.pool)
        .await;

        Ok(user)
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, COALESCE(email_normalized, LOWER(email)) AS email_normalized, name, role, 
                   COALESCE(status, 'ACTIVE') AS status,
                   COALESCE(auth_version, 0) AS auth_version,
                   email_verified_at, created_at, updated_at,
                   COALESCE(metadata, '{}'::jsonb) AS metadata
            FROM users
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let role_str: String = r.get("role");
            let role = str_to_user_role(&role_str)?;
            let user_id: Uuid = r.get("id");
            Ok(Some(User {
                id: domain::models::identifiers::UserId(user_id),
                email: r.get("email"),
                email_normalized: r.get("email_normalized"),
                name: r.get("name"),
                role,
                status: r.get("status"),
                auth_version: r.get("auth_version"),
                email_verified_at: r.get("email_verified_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let normalized = email.trim().to_lowercase();
        let row = sqlx::query(
            r#"
            SELECT id, email, COALESCE(email_normalized, LOWER(email)) AS email_normalized, name, role, 
                   COALESCE(status, 'ACTIVE') AS status,
                   COALESCE(auth_version, 0) AS auth_version,
                   email_verified_at, created_at, updated_at,
                   COALESCE(metadata, '{}'::jsonb) AS metadata
            FROM users
            WHERE email = $1 OR email_normalized = $2
            "#
        )
        .bind(email)
        .bind(normalized)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let role_str: String = r.get("role");
            let role = str_to_user_role(&role_str)?;
            let user_id: Uuid = r.get("id");
            Ok(Some(User {
                id: domain::models::identifiers::UserId(user_id),
                email: r.get("email"),
                email_normalized: r.get("email_normalized"),
                name: r.get("name"),
                role,
                status: r.get("status"),
                auth_version: r.get("auth_version"),
                email_verified_at: r.get("email_verified_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                metadata: r.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_identity_by_email(&self, email: &str) -> Result<Option<UserIdentity>, DomainError> {
        if let Some(user) = self.get_user_by_email(email).await? {
            // Check credentials table first, fallback to users.password_hash if legacy
            let cred_row = sqlx::query("SELECT secret_hash FROM credentials WHERE user_id = $1 AND kind = 'password' LIMIT 1")
                .bind(user.id.0)
                .fetch_optional(&self.pool)
                .await
                .ok()
                .flatten();

            let password_hash = if let Some(row) = cred_row {
                use sqlx::Row;
                row.get::<Option<String>, _>("secret_hash").unwrap_or_default()
            } else {
                String::new()
            };

            Ok(Some(UserIdentity {
                user,
                password_hash,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_user(&self, user: User) -> Result<User, DomainError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE users
            SET email = $1, email_normalized = $2, name = $3, role = $4, status = $5, email_verified_at = $6, metadata = $7, updated_at = NOW()
            WHERE id = $8
            "#
        )
        .bind(&user.email)
        .bind(&user.email_normalized)
        .bind(&user.name)
        .bind(user_role_to_str(&user.role))
        .bind(user.status)
        .bind(user.email_verified_at)
        .bind(&user.metadata)
        .bind(user.id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::UserNotFound);
        }

        Ok(user)
    }

    async fn create_verification_token(&self, token: VerificationToken) -> Result<VerificationToken, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO verification_tokens (id, user_id, token_hash, purpose, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(token.id)
        .bind(token.user_id)
        .bind(&token.token_hash)
        .bind(token_purpose_to_str(&token.purpose))
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(token)
    }

    async fn get_verification_token(&self, token_hash: &str) -> Result<Option<VerificationToken>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, token_hash, purpose, expires_at, created_at
            FROM verification_tokens
            WHERE token_hash = $1
            "#
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let purpose_str: String = r.get("purpose");
            let purpose = str_to_token_purpose(&purpose_str)?;
            Ok(Some(VerificationToken {
                id: r.get("id"),
                user_id: r.get("user_id"),
                token_hash: r.get("token_hash"),
                purpose,
                expires_at: r.get("expires_at"),
                created_at: r.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }
}
