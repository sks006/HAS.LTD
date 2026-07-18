#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    NotFound(String),
    InvalidState(String),
    InsufficientStock(String),
    ValidationFailed(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(message) => write!(f, "Not found: {message}"),
            Self::InvalidState(message) => write!(f, "Invalid state: {message}"),
            Self::InsufficientStock(message) => write!(f, "Insufficient stock: {message}"),
            Self::ValidationFailed(message) => write!(f, "Validation failed: {message}"),
        }
    }
}

impl std::error::Error for DomainError {}
