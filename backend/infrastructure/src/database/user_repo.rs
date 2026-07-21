// backend/infrastructure/src/database/user_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::auth::VerificationToken;
use domain::models::user::{User, UserIdentity};
use domain::repositories::UserRepository;

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
    async fn create_user(&self, _user: User, _password_hash: String) -> Result<User, DomainError> {
        unimplemented!()
    }

    async fn get_user_by_id(&self, _id: Uuid) -> Result<Option<User>, DomainError> {
        unimplemented!()
    }

    async fn get_user_by_email(&self, _email: &str) -> Result<Option<User>, DomainError> {
        unimplemented!()
    }

    async fn get_user_identity_by_email(&self, _email: &str) -> Result<Option<UserIdentity>, DomainError> {
        unimplemented!()
    }

    async fn update_user(&self, _user: User) -> Result<User, DomainError> {
        unimplemented!()
    }

    async fn create_verification_token(&self, _token: VerificationToken) -> Result<VerificationToken, DomainError> {
        unimplemented!()
    }

    async fn get_verification_token(&self, _token_hash: &str) -> Result<Option<VerificationToken>, DomainError> {
        unimplemented!()
    }
}
