// backend/domain/src/models/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::identifiers::UserId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserRole {
    Customer,
    Moderator,
    Admin,
}

impl sqlx::Type<sqlx::Postgres> for UserRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserRole {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "CUSTOMER" | "Customer" => Ok(UserRole::Customer),
            "MODERATOR" | "Moderator" => Ok(UserRole::Moderator),
            "ADMIN" | "Admin" => Ok(UserRole::Admin),
            other => Err(format!("Unknown user role: {other}").into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for UserRole {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = match self {
            UserRole::Customer => "CUSTOMER",
            UserRole::Moderator => "MODERATOR",
            UserRole::Admin => "ADMIN",
        };
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status_enum", rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub email_normalized: String,
    pub name: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub auth_version: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Legacy container maintained for backward compatibility during staged migration
#[derive(Debug, Clone)]
pub struct UserIdentity {
    pub user: User,
    pub password_hash: String,
}