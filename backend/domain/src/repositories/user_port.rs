// backend/domain/src/repositories/user_port.rs
use async_trait::async_trait;
use uuid::Uuid;
use crate::errors::DomainError;
use crate::models::user::{User, UserIdentity};
use crate::models::auth::VerificationToken;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: User, password_hash: String) -> Result<User, DomainError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn get_user_identity_by_email(&self, email: &str) -> Result<Option<UserIdentity>, DomainError>;
    async fn update_user(&self, user: User) -> Result<User, DomainError>;
    async fn create_verification_token(&self, token: VerificationToken) -> Result<VerificationToken, DomainError>;
    async fn get_verification_token(&self, token_hash: &str) -> Result<Option<VerificationToken>, DomainError>;
}
