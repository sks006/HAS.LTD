// backend/domain/src/repositories/inventory_port.rs

//! Defines the port for inventory management.
//! Inventory uses an append‑only ledger (event sourcing) and a read‑optimised snapshot
//! with optimistic concurrency control (OCC) to handle high‑contention checkouts.

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::inventory::{InventoryLedgerEntry, InventoryStockSnapshot};

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    // ---- Ledger (append‑only) ----

    

    /// Append a new inventory event (restock, reserve, release, fulfill).
    /// The ledger is the source of truth; it is never updated or deleted.
    /// Idempotency is enforced via `idempotency_key` on the entry.
    async fn append_ledger_entry(
        &self,
        entry: InventoryLedgerEntry,
    ) -> Result<InventoryLedgerEntry, DomainError>;

    /// Retrieve ledger entries for a variant, ordered by creation time.
    /// Used for audits, reconciliation, and rebuilding snapshots.
    /// Optional `limit`/`offset` for pagination (default: 100, 0).
    async fn get_ledger_entries(
        &self,
        variant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<InventoryLedgerEntry>, DomainError>;

    // ---- Stock Snapshot (read‑optimised projection) ----

    /// Get the current stock snapshot for a variant.
    /// The snapshot is a materialised view of the ledger, updated after each transaction.
    /// Returns `None` if the variant has no snapshot yet (e.g., newly created).
    async fn get_stock_snapshot(
        &self,
        variant_id: Uuid,
    ) -> Result<Option<InventoryStockSnapshot>, DomainError>;

    /// Update the stock snapshot using **optimistic concurrency control**.
    /// - The `version` field in the provided snapshot must match the current row version.
    /// - If they match, the update is applied and `version` is incremented.
    /// - If they differ, a `DomainError::ConcurrencyConflict` is returned.
    /// This prevents lost updates when two checkout threads try to update stock simultaneously.
    async fn update_stock_snapshot(
        &self,
        snapshot: InventoryStockSnapshot,
    ) -> Result<InventoryStockSnapshot, DomainError>;
    
    async fn get_stock_snapshots_by_variant_ids(&self, ids: &[Uuid]) -> Result<Vec<InventoryStockSnapshot>, DomainError>;
}