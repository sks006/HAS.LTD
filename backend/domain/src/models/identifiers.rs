// backend/domain/src/models/identifiers.rs
use std::fmt::{self, Display};
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            sqlx::Type,
        )]
        #[sqlx(transparent)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new(id: Uuid) -> Self {
                Self(id)
            }

            pub fn now_v7() -> Self {
                Self(Uuid::now_v7())
            }

            pub fn into_inner(self) -> Uuid {
                self.0
            }
        }

        impl Deref for $name {
            type Target = Uuid;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl AsRef<Uuid> for $name {
            fn as_ref(&self) -> &Uuid {
                &self.0
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                Self(id)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

define_id!(UserId);
define_id!(SessionId);
define_id!(RefreshTokenId);
define_id!(CredentialId);
define_id!(MfaCredentialId);
define_id!(AuditLogId);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_id_surface<T>()
    where
        T: sqlx::Type<sqlx::Postgres>
            + for<'r> sqlx::Decode<'r, sqlx::Postgres>
            + for<'q> sqlx::Encode<'q, sqlx::Postgres>
            + From<Uuid>
            + Into<Uuid>
            + AsRef<Uuid>
            + Serialize
            + for<'de> Deserialize<'de>
            + Display
            + std::fmt::Debug
            + Clone
            + Copy
            + PartialEq
            + Eq
            + std::hash::Hash,
    {
    }

    #[test]
    fn test_define_id_sqlx_surface_and_traits() {
        assert_id_surface::<UserId>();
        assert_id_surface::<SessionId>();
        assert_id_surface::<RefreshTokenId>();
        assert_id_surface::<CredentialId>();
        assert_id_surface::<MfaCredentialId>();
        assert_id_surface::<AuditLogId>();

        let raw_uuid = Uuid::new_v4();

        // 1. UserId
        let user_id = UserId::new(raw_uuid);
        assert_eq!(*user_id.as_ref(), raw_uuid);
        let into_uuid: Uuid = user_id.into();
        assert_eq!(into_uuid, raw_uuid);
        let serialized = serde_json::to_string(&user_id).unwrap();
        let deserialized: UserId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, user_id);
        let _q1 = sqlx::query_scalar::<sqlx::Postgres, UserId>("SELECT $1::uuid").bind(user_id);

        // 2. SessionId
        let session_id = SessionId::new(raw_uuid);
        assert_eq!(*session_id.as_ref(), raw_uuid);
        let _q2 = sqlx::query_scalar::<sqlx::Postgres, SessionId>("SELECT $1::uuid").bind(session_id);

        // 3. RefreshTokenId
        let refresh_token_id = RefreshTokenId::new(raw_uuid);
        assert_eq!(*refresh_token_id.as_ref(), raw_uuid);
        let _q3 = sqlx::query_scalar::<sqlx::Postgres, RefreshTokenId>("SELECT $1::uuid").bind(refresh_token_id);

        // 4. CredentialId
        let cred_id = CredentialId::new(raw_uuid);
        assert_eq!(*cred_id.as_ref(), raw_uuid);
        let _q4 = sqlx::query_scalar::<sqlx::Postgres, CredentialId>("SELECT $1::uuid").bind(cred_id);

        // 5. MfaCredentialId
        let mfa_id = MfaCredentialId::new(raw_uuid);
        assert_eq!(*mfa_id.as_ref(), raw_uuid);
        let _q5 = sqlx::query_scalar::<sqlx::Postgres, MfaCredentialId>("SELECT $1::uuid").bind(mfa_id);

        // 6. AuditLogId
        let audit_id = AuditLogId::new(raw_uuid);
        assert_eq!(*audit_id.as_ref(), raw_uuid);
        let _q6 = sqlx::query_scalar::<sqlx::Postgres, AuditLogId>("SELECT $1::uuid").bind(audit_id);
    }
}

