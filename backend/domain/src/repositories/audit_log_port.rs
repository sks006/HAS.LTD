// backend/domain/src/repositories/audit_log_port.rs
use async_trait::async_trait;
use crate::errors::DomainError;
use crate::models::identifiers::UserId;
use crate::models::audit::AuditEvent;

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn record(&self, event: AuditEvent) -> Result<AuditEvent, DomainError>;
    async fn list_by_user_id(&self, user_id: UserId, limit: u32, offset: u32) -> Result<Vec<AuditEvent>, DomainError>;
    async fn list_by_event_type(&self, event_type: &str, limit: u32, offset: u32) -> Result<Vec<AuditEvent>, DomainError>;
}
