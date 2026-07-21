// backend/infrastructure/src/database/inventory_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::inventory::{InventoryLedgerEntry, InventoryStockSnapshot};
use domain::repositories::InventoryRepository;

pub struct PgInventoryRepository {
    pub pool: PgPool,
}

impl PgInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InventoryRepository for PgInventoryRepository {
    async fn append_ledger_entry(
        &self,
        _entry: InventoryLedgerEntry,
    ) -> Result<InventoryLedgerEntry, DomainError> {
        unimplemented!()
    }

    async fn get_ledger_entries(
        &self,
        _variant_id: Uuid,
        _limit: Option<u32>,
        _offset: Option<u32>,
    ) -> Result<Vec<InventoryLedgerEntry>, DomainError> {
        unimplemented!()
    }

    async fn get_stock_snapshot(
        &self,
        _variant_id: Uuid,
    ) -> Result<Option<InventoryStockSnapshot>, DomainError> {
        unimplemented!()
    }

    async fn update_stock_snapshot(
        &self,
        _snapshot: InventoryStockSnapshot,
    ) -> Result<InventoryStockSnapshot, DomainError> {
        unimplemented!()
    }

    async fn get_stock_snapshots_by_variant_ids(
        &self,
        _ids: &[Uuid],
    ) -> Result<Vec<InventoryStockSnapshot>, DomainError> {
        unimplemented!()
    }
}
