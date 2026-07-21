// backend/domain/src/repositories/checkout_port.rs
use async_trait::async_trait;
use crate::errors::DomainError;
use crate::models::inventory::{InventoryLedgerEntry, InventoryStockSnapshot};
use crate::models::order::{Order, OrderItem};

#[async_trait]
pub trait CheckoutTransactionPort: Send + Sync {
    /// Atomically commits the entire checkout state in a single database transaction.
    async fn commit_checkout_state(
        &self,
        order: Order,
        order_items: Vec<OrderItem>,
        ledger_entries: Vec<InventoryLedgerEntry>,
        updated_snapshots: Vec<InventoryStockSnapshot>,
    ) -> Result<(), DomainError>;
}
