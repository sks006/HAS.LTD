// backend/infrastructure/src/email/in_memory_adapter.rs

use std::sync::Mutex;
use async_trait::async_trait;
use domain::ports::email_port::{EmailError, EmailMessage, EmailPort};

#[derive(Debug, Default)]
pub struct InMemoryEmailAdapter {
    sent: Mutex<Vec<EmailMessage>>,
}

impl InMemoryEmailAdapter {
    pub fn new() -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
        }
    }

    /// Drains and returns all captured email messages.
    pub fn drain(&self) -> Vec<EmailMessage> {
        let mut lock = self.sent.lock().expect("in-memory email lock");
        std::mem::take(&mut *lock)
    }

    /// Returns a clone of the last captured email message, if any.
    pub fn last(&self) -> Option<EmailMessage> {
        let lock = self.sent.lock().expect("in-memory email lock");
        lock.last().cloned()
    }

    /// Returns a clone of all captured email messages.
    pub fn all(&self) -> Vec<EmailMessage> {
        let lock = self.sent.lock().expect("in-memory email lock");
        lock.clone()
    }

    /// Returns the number of captured email messages.
    pub fn count(&self) -> usize {
        let lock = self.sent.lock().expect("in-memory email lock");
        lock.len()
    }

    /// Clears all captured email messages.
    pub fn clear(&self) {
        let mut lock = self.sent.lock().expect("in-memory email lock");
        lock.clear();
    }
}

#[async_trait]
impl EmailPort for InMemoryEmailAdapter {
    async fn send(&self, msg: EmailMessage) -> Result<(), EmailError> {
        tracing::info!(
            to = %msg.to,
            subject = %msg.subject,
            template_id = %msg.template_id,
            variables = %msg.variables,
            "InMemoryEmailAdapter: email captured"
        );
        let mut lock = self.sent.lock().expect("in-memory email lock");
        lock.push(msg);
        Ok(())
    }
}
