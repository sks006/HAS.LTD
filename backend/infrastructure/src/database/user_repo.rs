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
    match role {
        UserRole::Customer => "Customer",
        UserRole::Admin => "Admin",
        UserRole::Moderator => "Moderator",
    }
}

fn str_to_user_role(role_str: &str) -> Result<UserRole, DomainError> {
    match role_str {
        "Customer" => Ok(UserRole::Customer),
        "Admin" => Ok(UserRole::Admin),
        "Moderator" => Ok(UserRole::Moderator),
        _ => Err(DomainError::InvalidEnumValue(format!("Unknown user role: {}", role_str))),
    }
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
            INSERT INTO users (id, email, password_hash, name, role, is_active, deleted_at, email_verified_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&password_hash)
        .bind(&user.name)
        .bind(user_role_to_str(&user.role))
        .bind(user.is_active)
        .bind(user.deleted_at)
        .bind(user.email_verified_at)
        .bind(user.created_at)
        .bind(user.updated_at)
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

        Ok(user)
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, role, is_active, deleted_at, email_verified_at, created_at, updated_at
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
            Ok(Some(User {
                id: r.get("id"),
                email: r.get("email"),
                name: r.get("name"),
                role,
                is_active: r.get("is_active"),
                deleted_at: r.get("deleted_at"),
                email_verified_at: r.get("email_verified_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, role, is_active, deleted_at, email_verified_at, created_at, updated_at
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let role_str: String = r.get("role");
            let role = str_to_user_role(&role_str)?;
            Ok(Some(User {
                id: r.get("id"),
                email: r.get("email"),
                name: r.get("name"),
                role,
                is_active: r.get("is_active"),
                deleted_at: r.get("deleted_at"),
                email_verified_at: r.get("email_verified_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user_identity_by_email(&self, email: &str) -> Result<Option<UserIdentity>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, email, name, role, is_active, deleted_at, email_verified_at, created_at, updated_at, password_hash
            FROM users
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let role_str: String = r.get("role");
            let role = str_to_user_role(&role_str)?;
            let user = User {
                id: r.get("id"),
                email: r.get("email"),
                name: r.get("name"),
                role,
                is_active: r.get("is_active"),
                deleted_at: r.get("deleted_at"),
                email_verified_at: r.get("email_verified_at"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            };
            let password_hash: String = r.get("password_hash");
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
            SET email = $1, name = $2, role = $3, is_active = $4, deleted_at = $5, email_verified_at = $6, updated_at = NOW()
            WHERE id = $7
            "#
        )
        .bind(&user.email)
        .bind(&user.name)
        .bind(user_role_to_str(&user.role))
        .bind(user.is_active)
        .bind(user.deleted_at)
        .bind(user.email_verified_at)
        .bind(user.id)
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
