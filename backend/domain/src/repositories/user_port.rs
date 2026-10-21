// backend/domain/src/repositories/user_port.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::errors::DomainError;
use crate::models::user::{User, UserIdentity, UserRole, UserStatus};
use crate::models::identifiers::UserId;
use crate::models::auth::VerificationToken;

#[async_trait]
pub trait UserRepository: Send + Sync {
    // Existing methods preserved for backward compatibility
    async fn create_user(&self, user: User, password_hash: String) -> Result<User, DomainError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn get_user_identity_by_email(&self, email: &str) -> Result<Option<UserIdentity>, DomainError>;
    async fn update_user(&self, user: User) -> Result<User, DomainError>;
    async fn create_verification_token(&self, token: VerificationToken) -> Result<VerificationToken, DomainError>;
    async fn get_verification_token(&self, token_hash: &str) -> Result<Option<VerificationToken>, DomainError>;

    // Extended Phase 1 methods (with default fallback stubs until adapter implementation in Phase 2)
    async fn find_by_normalized_email(&self, normalized_email: &str) -> Result<Option<User>, DomainError> {
        self.get_user_by_email(normalized_email).await
    }

    async fn update_status(&self, _id: UserId, _status: UserStatus) -> Result<(), DomainError> {
        Err(DomainError::Database("Not implemented yet".to_string()))
    }

    async fn update_role(&self, _id: UserId, _role: UserRole) -> Result<(), DomainError> {
        Err(DomainError::Database("Not implemented yet".to_string()))
    }

    async fn bump_auth_version(&self, _id: UserId) -> Result<i64, DomainError> {
        Err(DomainError::Database("Not implemented yet".to_string()))
    }

    async fn anonymize_user(&self, _id: UserId) -> Result<(), DomainError> {
        Err(DomainError::Database("Not implemented yet".to_string()))
    }
}
