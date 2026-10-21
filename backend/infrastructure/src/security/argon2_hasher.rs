use domain::repositories::HasherPort;
use domain::errors::DomainError;

pub struct Argon2Hasher;

impl HasherPort for Argon2Hasher {
    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        // Simple mock hashing for development
        Ok(format!("argon2id$mock${}", password))
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, DomainError> {
        let expected = format!("argon2id$mock${}", password);
        Ok(hash == expected)
    }
}
