// backend/domain/src/ports/email_port.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub template_id: String,
    pub variables: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailError {
    Transient(String),
    Permanent(String),
    Invalid(String),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transient(msg) => write!(f, "Transient email error: {}", msg),
            Self::Permanent(msg) => write!(f, "Permanent email error: {}", msg),
            Self::Invalid(msg) => write!(f, "Invalid email error: {}", msg),
        }
    }
}

impl std::error::Error for EmailError {}

#[async_trait]
pub trait EmailPort: Send + Sync {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError>;
}
