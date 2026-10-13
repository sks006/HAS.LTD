// backend/domain/src/models/identifiers.rs
use std::fmt::{self, Display};
use std::ops::Deref;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, sqlx::Type)]
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
