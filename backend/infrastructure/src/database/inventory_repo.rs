//! PostgreSQL adapter for the InventoryRepository port.
//! Handles inventory ledger (append‑only) and stock snapshots (read‑optimised with OCC).

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use domain::errors::DomainError;
use domain::models::inventory::{InventoryLedgerEntry, InventoryStockSnapshot, LedgerAction};
use domain::repositories::InventoryRepository;

/// Physical PostgreSQL implementation of the inventory repository.
pub struct SqlxInventoryRepository {
    pool: PgPool,
}

impl SqlxInventoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ---- Enum mapping to PostgreSQL ENUM ----
// Derive `sqlx::Type` to map directly to the database enum.
// No string conversions at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "ledger_action_enum", rename_all = "PascalCase")]
pub enum LedgerActionDb {
    Restock,
    Reserve,
    Release,
    Fulfill,
}

impl From<LedgerAction> for LedgerActionDb {
    fn from(a: LedgerAction) -> Self {
        match a {
            LedgerAction::Restock => LedgerActionDb::Restock,
            LedgerAction::Reserve => LedgerActionDb::Reserve,
            LedgerAction::Release => LedgerActionDb::Release,
            LedgerAction::Fulfill => LedgerActionDb::Fulfill,
        }
    }
}

impl From<LedgerActionDb> for LedgerAction {
    fn from(a: LedgerActionDb) -> Self {
        match a {
            LedgerActionDb::Restock => LedgerAction::Restock,
            LedgerActionDb::Reserve => LedgerAction::Reserve,
            LedgerActionDb::Release => LedgerAction::Release,
            LedgerActionDb::Fulfill => LedgerAction::Fulfill,
        }
    }
}

#[async_trait]
impl InventoryRepository for SqlxInventoryRepository {
    async fn append_ledger_entry(
        &self,
        entry: InventoryLedgerEntry,
    ) -> Result<InventoryLedgerEntry, DomainError> {
        let row = sqlx::query(
            r#"
        INSERT INTO inventory_ledger (
            id, variant_id, action, quantity_change,
            reference_id, idempotency_key, created_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, variant_id, action, quantity_change, reference_id, idempotency_key, created_at
        "#,
        )
        .bind(entry.id)
        .bind(entry.variant_id)
        .bind(LedgerActionDb::from(entry.action))
        .bind(entry.quantity_change)
        .bind(entry.reference_id)
        .bind(&entry.idempotency_key)
        .bind(entry.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(InventoryLedgerEntry {
            id: row.get("id"),
            variant_id: row.get("variant_id"),
            action: row.get::<LedgerActionDb, _>("action").into(),
            quantity_change: row.get("quantity_change"),
            reference_id: row.get("reference_id"),
            idempotency_key: row.get("idempotency_key"),
            created_at: row.get("created_at"),
        })
    }
    async fn get_ledger_entries(
        &self,
        variant_id: Uuid,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<InventoryLedgerEntry>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, variant_id, action, quantity_change, reference_id, idempotency_key, created_at
            FROM inventory_ledger
            WHERE variant_id = $1
            ORDER BY created_at DESC
            LIMIT COALESCE($2, 100) OFFSET COALESCE($3, 0)
            "#,
        )
        .bind(variant_id)
        .bind(limit.map(|l| l as i64))
        .bind(offset.map(|o| o as i64))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| InventoryLedgerEntry {
                id: r.get("id"),
                variant_id: r.get("variant_id"),
                action: r.get::<LedgerActionDb, _>("action").into(),
                quantity_change: r.get("quantity_change"),
                reference_id: r.get("reference_id"),
                idempotency_key: r.get("idempotency_key"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn get_stock_snapshot(
        &self,
        variant_id: Uuid,
    ) -> Result<Option<InventoryStockSnapshot>, DomainError> {
        let row = sqlx::query(
            "SELECT variant_id, stock, reserved_stock, available_stock, version, created_at, updated_at FROM inventory_stock_snapshots WHERE variant_id = $1",
        )
        .bind(variant_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(|r| InventoryStockSnapshot {
            variant_id: r.get("variant_id"),
            stock: r.get("stock"),
            reserved_stock: r.get("reserved_stock"),
            available_stock: r.get("available_stock"),
            version: r.get("version"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

  async fn update_stock_snapshot(
    &self,
    snapshot: InventoryStockSnapshot,
) -> Result<InventoryStockSnapshot, DomainError> {
    let row = sqlx::query(
        r#"
        UPDATE inventory_stock_snapshots
        SET
            stock = $1,
            reserved_stock = $2,
            available_stock = $3,
            version = version + 1,
            updated_at = NOW()
        WHERE variant_id = $4
          AND version = $5
        RETURNING variant_id, stock, reserved_stock, available_stock, version, created_at, updated_at
        "#,
    )
    .bind(snapshot.stock)
    .bind(snapshot.reserved_stock)
    .bind(snapshot.available_stock)
    .bind(snapshot.variant_id)
    .bind(snapshot.version)
    .fetch_optional(&self.pool)
    .await
    .map_err(|e| DomainError::Database(e.to_string()))?;

    match row {
        Some(r) => Ok(InventoryStockSnapshot {
            variant_id: r.get("variant_id"),
            stock: r.get("stock"),
            reserved_stock: r.get("reserved_stock"),
            available_stock: r.get("available_stock"),
            version: r.get("version"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }),
        None => Err(DomainError::ConcurrencyConflict {
            message: format!(
                "Stock snapshot for variant {} changed during update (expected version {})",
                snapshot.variant_id, snapshot.version
            ),
        }),
    }
}

    async fn get_stock_snapshots_by_variant_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<InventoryStockSnapshot>, DomainError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT
                variant_id, stock, reserved_stock, available_stock,
                version, created_at, updated_at
            FROM inventory_stock_snapshots
            WHERE variant_id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| InventoryStockSnapshot {
                variant_id: r.get("variant_id"),
                stock: r.get("stock"),
                reserved_stock: r.get("reserved_stock"),
                available_stock: r.get("available_stock"),
                version: r.get("version"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            })
            .collect())
    }
}
