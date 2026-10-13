use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount_minor_units: i32,
    pub currency: String,
    pub status: String,
    pub client_secret: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementProof {
    pub transaction_id: String,
    pub provider: String,
    pub settled_at: DateTime<Utc>,
}
