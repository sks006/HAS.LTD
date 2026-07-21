// backend/domain/src/models/inventory.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LedgerAction {
    Restock,  // Positive quantity
    Reserve,  // Negative quantity (Checkout hold)
    Release,  // Positive quantity (Cart abandonment)
    Fulfill,  // Administrative status change (Quantity invariant)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryLedgerEntry {
    pub id: Uuid,
    pub variant_id: Uuid,
    pub action: LedgerAction,
    pub quantity_change: i32,
    pub reference_id: Option<Uuid>, 
    pub idempotency_key: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryStockSnapshot {
    pub variant_id: Uuid,
    pub stock: i32,
    pub reserved_stock: i32,
    pub available_stock: i32,
    pub version: i32, // MANDATORY: For optimistic concurrency
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,  // <-- tracks last refresh from ledger
}