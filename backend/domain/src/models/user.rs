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

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Customer => "CUSTOMER",
            UserRole::Moderator => "MODERATOR",
            UserRole::Admin => "ADMIN",
        }
    }
}

impl AsRef<str> for UserRole {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for UserRole {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_ascii_uppercase().as_str() {
            "CUSTOMER" => Ok(UserRole::Customer),
            "MODERATOR" => Ok(UserRole::Moderator),
            "ADMIN" => Ok(UserRole::Admin),
            other => Err(format!("Unknown user role: {other}")),
        }
    }
}

impl TryFrom<String> for UserRole {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl std::str::FromStr for UserRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl sqlx::Type<sqlx::Postgres> for UserRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserRole {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        UserRole::try_from(s).map_err(|e| e.into())
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for UserRole {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.as_str();
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserStatus {
    Active,
    Suspended,
    Deleted,
    Pending,
}

impl UserStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserStatus::Active => "ACTIVE",
            UserStatus::Suspended => "SUSPENDED",
            UserStatus::Deleted => "DELETED",
            UserStatus::Pending => "PENDING",
        }
    }
}

impl AsRef<str> for UserStatus {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for UserStatus {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_ascii_uppercase().as_str() {
            "ACTIVE" => Ok(UserStatus::Active),
            "SUSPENDED" => Ok(UserStatus::Suspended),
            "DELETED" => Ok(UserStatus::Deleted),
            "PENDING" => Ok(UserStatus::Pending),
            other => Err(format!("Unknown user status: {other}")),
        }
    }
}

impl TryFrom<String> for UserStatus {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

impl std::str::FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl sqlx::Type<sqlx::Postgres> for UserStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }

    fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
            || *ty == sqlx::postgres::PgTypeInfo::with_name("user_status_enum")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for UserStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        UserStatus::try_from(s).map_err(|e| e.into())
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for UserStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let s = self.as_str();
        <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_role_conversions_and_sqlx_surface() {
        assert_eq!(UserRole::try_from("customer").unwrap(), UserRole::Customer);
        assert_eq!(UserRole::try_from("CUSTOMER").unwrap(), UserRole::Customer);
        assert_eq!(UserRole::try_from("moderator").unwrap(), UserRole::Moderator);
        assert_eq!(UserRole::try_from("MODERATOR").unwrap(), UserRole::Moderator);
        assert_eq!(UserRole::try_from("admin").unwrap(), UserRole::Admin);
        assert_eq!(UserRole::try_from("ADMIN").unwrap(), UserRole::Admin);
        assert!(UserRole::try_from("INVALID").is_err());

        assert_eq!(UserRole::Customer.as_str(), "CUSTOMER");
        assert_eq!(UserRole::Customer.to_string(), "CUSTOMER");

        // Verify sqlx traits compile and bind properly for Postgres
        let _q = sqlx::query_scalar::<sqlx::Postgres, UserRole>("SELECT $1::varchar").bind(UserRole::Admin);
    }

    #[test]
    fn test_user_status_conversions_and_sqlx_surface() {
        assert_eq!(UserStatus::try_from("active").unwrap(), UserStatus::Active);
        assert_eq!(UserStatus::try_from("ACTIVE").unwrap(), UserStatus::Active);
        assert_eq!(UserStatus::try_from("suspended").unwrap(), UserStatus::Suspended);
        assert_eq!(UserStatus::try_from("SUSPENDED").unwrap(), UserStatus::Suspended);
        assert_eq!(UserStatus::try_from("deleted").unwrap(), UserStatus::Deleted);
        assert_eq!(UserStatus::try_from("DELETED").unwrap(), UserStatus::Deleted);
        assert_eq!(UserStatus::try_from("pending").unwrap(), UserStatus::Pending);
        assert_eq!(UserStatus::try_from("PENDING").unwrap(), UserStatus::Pending);
        assert!(UserStatus::try_from("UNKNOWN").is_err());

        assert_eq!(UserStatus::Active.as_str(), "ACTIVE");
        assert_eq!(UserStatus::Active.to_string(), "ACTIVE");

        // Verify sqlx traits compile and bind properly for Postgres
        let _q = sqlx::query_scalar::<sqlx::Postgres, UserStatus>("SELECT $1::varchar").bind(UserStatus::Active);
    }
}