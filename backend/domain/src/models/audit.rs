// backend/domain/src/models/audit.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::models::identifiers::{AuditLogId, UserId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditEvent {
    pub id: AuditLogId,
    pub user_id: Option<UserId>,
    pub event: String,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
