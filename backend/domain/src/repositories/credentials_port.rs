// backend/domain/src/repositories/credentials_port.rs
use async_trait::async_trait;
use crate::errors::DomainError;
use crate::models::identifiers::{CredentialId, UserId};
use crate::models::credentials::{Credentials, CredentialKind};

#[async_trait]
pub trait CredentialsRepository: Send + Sync {
    async fn create(&self, credentials: Credentials) -> Result<Credentials, DomainError>;
    async fn find_by_user_id_and_kind(
        &self,
        user_id: UserId,
        kind: CredentialKind,
        provider: Option<&str>,
    ) -> Result<Option<Credentials>, DomainError>;
    async fn find_by_provider_subject(
        &self,
        provider: &str,
        subject: &str,
    ) -> Result<Option<Credentials>, DomainError>;
    async fn update_secret_hash(
        &self,
        credential_id: CredentialId,
        new_hash: &str,
    ) -> Result<(), DomainError>;
    async fn delete_by_user_id(&self, user_id: UserId) -> Result<(), DomainError>;
}
