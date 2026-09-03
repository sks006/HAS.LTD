//! PostgreSQL adapter for the CheckoutTransactionPort.
//! This adapter atomically commits an entire checkout state within a single
//! database transaction. It uses vectorised inserts and explicit error classification
//! to avoid panics, N+1 round‑trips, and lost error context.

// ---- Imports ----
// `async_trait` allows async methods in traits (Rust 1.75+).
// `sqlx` provides PostgreSQL connection, transaction, and query builder.
// `tracing::error` logs errors without panicking.
use async_trait::async_trait;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::error;

// Domain imports: error types, aggregates, and the port trait.
use domain::errors::DomainError;
use domain::models::order::{Order, OrderItem, OrderState};
use domain::models::inventory::{InventoryLedgerEntry, InventoryStockSnapshot, LedgerAction};
use domain::repositories::CheckoutTransactionPort;

// ---- Physical Adapter Struct ----
/// Holds a PostgreSQL connection pool. Stateless – all state is in the transaction.
pub struct SqlxCheckoutAdapter {
    pool: PgPool, // Clonable, thread‑safe connection pool (Arc internally).
}

impl SqlxCheckoutAdapter {
    /// Constructor – injects the pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// ---- Trait Implementation ----
#[async_trait]
impl CheckoutTransactionPort for SqlxCheckoutAdapter {
    /// The only method: commits the entire checkout state atomically.
    async fn commit_checkout_state(
        &self,
        order: Order,                            // The order aggregate (version = 1).
        order_items: Vec<OrderItem>,             // Immutable snapshots of purchased variants.
        ledger_entries: Vec<InventoryLedgerEntry>, // Append‑only inventory events.
        updated_snapshots: Vec<InventoryStockSnapshot>, // New stock states (with OCC version).
    ) -> Result<(), DomainError> {

        // ---- MARKER 1: Acquire Transaction ----
        // `self.pool.begin()` starts a PostgreSQL transaction (default: READ COMMITTED).
        // `mut tx` holds the transaction handle. If not committed, it will roll back on drop.
        let mut tx = self.pool
            .begin()
            .await
            // If pool acquisition fails (e.g., no connections), map to DomainError::Database.
            .map_err(|e| {
                error!("Failed to start transaction: {}", e);
                DomainError::Database(e.to_string())
            })?;

        // ---- MARKER 2: Insert Order (with Safe Serialisation) ----
        // Serialise the `Address` structs to `serde_json::Value` (JSONB in PostgreSQL).
        // `map_err` converts any serialisation error into `DomainError::Serialization`.
        // This is a fallible operation; if it fails, we return early and `tx` drops → ROLLBACK.
        let shipping_json = serde_json::to_value(&order.shipping_address)
            .map_err(|e| {
                error!("Failed to serialise shipping_address: {}", e);
                DomainError::Serialization(e.to_string())
            })?;
        let billing_json = serde_json::to_value(&order.billing_address)
            .map_err(|e| {
                error!("Failed to serialise billing_address: {}", e);
                DomainError::Serialization(e.to_string())
            })?;

        // Build the INSERT query for the orders table.
        // We use `.bind()` to safely pass parameters – sqlx validates types at compile time.
        // The `if let Err(e)` pattern lets us inspect the error for idempotency collisions.
        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO orders (
                id, user_id, state, currency,
                shipping_address, billing_address,
                total_minor_units, version, idempotency_key,
                created_at, updated_at
            )
            VALUES ($1, $2, $3::order_state_enum, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(order.id)                         // $1
        .bind(order.user_id)                    // $2
        .bind(order.state.as_str())             // $3 – enum to string via helper
        .bind(&order.currency)                  // $4
        .bind(shipping_json)                    // $5 – JSONB
        .bind(billing_json)                     // $6 – JSONB
        .bind(order.total_minor_units)          // $7
        .bind(order.version)                    // $8 – initial version = 1
        .bind(&order.idempotency_key)           // $9 – unique key for idempotency
        .bind(order.created_at)                 // $10
        .bind(order.updated_at)                 // $11
        .execute(&mut *tx)                      // Execute inside the transaction.
        .await
        {
            // ---- Idempotency Collision Detection ----
            // PostgreSQL error code 23505 means `unique_violation`.
            // If the error message mentions `idempotency_key`, we know it's a duplicate order.
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    let msg = db_err.message();
                    if msg.contains("idempotency_key") {
                        // Return a specific error so the orchestrator can return the existing order.
                        return Err(DomainError::IdempotencyCollision(order.idempotency_key.clone()));
                    }
                }
            }
            // Otherwise, propagate as a generic database error.
            error!("Failed to insert order {}: {}", order.id, e);
            return Err(DomainError::Database(e.to_string()));
        }

        // ---- MARKER 3: Batch‑Insert Order Items (Pre‑Serialised) ----
        // PHASE 1: Allocate an intermediate vector with exact capacity (no reallocation).
        let mut items_with_attrs = Vec::with_capacity(order_items.len());

        // PHASE 2: Serialise attributes in the main async body – safe with `?`.
        // This loop consumes `order_items` (moves them into the vector).
        // If serialisation fails, we return `DomainError::Serialization` and roll back.
        for item in order_items {
            let attrs = serde_json::to_value(&item.attributes)
                .map_err(|e| {
                    error!("Failed to serialise attributes for item {}: {}", item.id, e);
                    DomainError::Serialization(e.to_string())
                })?;
            // PHASE 3: Push a tuple `(item, serialized_attributes)`.
            // `item` is moved into the vector; `attrs` is a `serde_json::Value`.
            items_with_attrs.push((item, attrs));
        }

        // PHASE 4: Clean injection – the closure does NO fallible work.
        if !items_with_attrs.is_empty() {
            // Build a single `INSERT INTO order_items (...)` with multiple rows.
            // `QueryBuilder` generates a multi‑value INSERT string.
            let mut query_builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO order_items (id, order_id, variant_id, quantity, product_name, sku, price_minor_units, currency, thumbnail_url, attributes) "
            );
            // `push_values` takes an iterator and a closure.
            // The closure binds each field – all values are already pre‑computed.
            query_builder.push_values(items_with_attrs.iter(), |mut b, (item, attrs)| {
                b.push_bind(item.id)
                 .push_bind(item.order_id)
                 .push_bind(item.variant_id)
                 .push_bind(item.quantity)
                 .push_bind(&item.product_name)
                 .push_bind(&item.sku)
                 .push_bind(item.price_minor_units)
                 .push_bind(&item.currency)
                 .push_bind(&item.thumbnail_url)
                 .push_bind(attrs); // `attrs` is already serialised – no unwrap needed.
            });

            // Build the final query and execute it within the transaction.
            // If this fails (e.g., foreign key violation), the error propagates.
            query_builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to batch-insert order items: {}", e);
                    DomainError::Database(e.to_string())
                })?;
        }

        // ---- MARKER 4: Batch‑Insert Ledger Entries ----
        // No serialisation needed – all fields are primitives or UUIDs.
        if !ledger_entries.is_empty() {
            let mut query_builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO inventory_ledger (id, variant_id, action, quantity_change, reference_id, idempotency_key, created_at) "
            );
            query_builder.push_values(ledger_entries.iter(), |mut b, entry| {
                b.push_bind(entry.id)
                 .push_bind(entry.variant_id)
                 .push_bind(entry.action.as_str()) // enum → string
                 .push_bind(entry.quantity_change)
                 .push_bind(entry.reference_id)
                 .push_bind(&entry.idempotency_key)
                 .push_bind(entry.created_at);
            });

            query_builder
                .build()
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    error!("Failed to batch-insert ledger entries: {}", e);
                    DomainError::Database(e.to_string())
                })?;
        }

        // ---- MARKER 5: OCC Loop for Stock Snapshots ----
        // Each snapshot is updated conditionally on `version`.
        // We iterate – this loop does NOT cause N+1 because each is a single UPDATE,
        // and they are fast (indexed by `variant_id`). For extreme concurrency, we could
        // batch them with `UNION`, but this is acceptable for typical cart sizes.
        for snapshot in updated_snapshots {
            let variant_id = snapshot.variant_id;
            let expected_version = snapshot.version; // version the domain read earlier.

            // Execute an UPDATE with a `WHERE version = $5` clause.
            // If the version has changed (another transaction updated it),
            // `rows_affected` will be 0.
            let rows_affected = sqlx::query(
                r#"
                UPDATE inventory_stock_snapshots
                SET
                    stock = $1,
                    reserved_stock = $2,
                    available_stock = $3,
                    version = version + 1,          // increment on success
                    updated_at = NOW()
                WHERE variant_id = $4
                  AND version = $5                  // OCC condition
                "#,
            )
            .bind(snapshot.stock)
            .bind(snapshot.reserved_stock)
            .bind(snapshot.available_stock)
            .bind(variant_id)
            .bind(expected_version)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                error!("Failed to update snapshot for variant {}: {}", variant_id, e);
                DomainError::Database(e.to_string())
            })?
            .rows_affected();

            // If no rows updated, the version was stale – raise a concurrency conflict.
            // This immediately returns `Err`, dropping `tx` and rolling back all changes.
            if rows_affected == 0 {
                return Err(DomainError::ConcurrencyConflict {
                    message: format!(
                        "Stock snapshot for variant {} changed since checkout (expected version {})",
                        variant_id, expected_version
                    ),
                });
            }
        }

        // ---- MARKER 6: Commit ----
        // All inserts and updates succeeded. Commit the transaction.
        // If commit fails (e.g., network error), `tx` is dropped and a rollback occurs.
        tx.commit()
            .await
            .map_err(|e| {
                error!("Failed to commit transaction: {}", e);
                DomainError::Database(e.to_string())
            })?;

        Ok(()) // Success – the entire checkout state is now durably stored.
    }
}

// ---- Enum to String Helpers ----
// These are required because the database stores enums as strings.
// Each variant maps to a `&'static str` that matches the CHECK constraint in SQL.

trait ToDbString {
    fn as_str(&self) -> &'static str;
}

impl ToDbString for OrderState {
    fn as_str(&self) -> &'static str {
        match self {
            OrderState::Pending => "Pending",
            OrderState::Reserved => "Reserved",
            OrderState::Paid => "Paid",
            OrderState::Shipped => "Shipped",
            OrderState::Cancelled => "Cancelled",
        }
    }
}

impl ToDbString for LedgerAction {
    fn as_str(&self) -> &'static str {
        match self {
            LedgerAction::Restock => "Restock",
            LedgerAction::Reserve => "Reserve",
            LedgerAction::Release => "Release",
            LedgerAction::Fulfill => "Fulfill",
        }
    }
}