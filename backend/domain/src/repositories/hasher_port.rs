pub trait HasherPort: Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, crate::errors::DomainError>;
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, crate::errors::DomainError>;
}
