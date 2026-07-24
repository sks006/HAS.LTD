// backend/domain/src/services/checkout.rs

use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

use crate::errors::DomainError;
use crate::models::order::{Order, OrderItem, OrderState, Address};
use crate::models::inventory::{InventoryLedgerEntry, LedgerAction};
use crate::repositories::{
    OrderRepository,
    InventoryRepository,
    ProductRepository,
    CheckoutTransactionPort,
};

// Input DTOs remain the same
pub struct CreateOrderInput {
    pub user_id: Option<Uuid>,
    pub items: Vec<OrderItemInput>,
    pub shipping_address: Address,
    pub billing_address: Address,
    pub idempotency_key: String,
    pub currency: String,
}

pub struct OrderItemInput {
    pub variant_id: Uuid,
    pub quantity: i32,
}

/// Main orchestrator – now depends on the transaction port
pub struct CheckoutService<R, I, P, T>
where
    R: OrderRepository,
    I: InventoryRepository,
    P: ProductRepository,
    T: CheckoutTransactionPort,
{
    order_repo: R,
    inventory_repo: I,
    product_repo: P,
    transaction_port: T,
}

impl<R, I, P, T> CheckoutService<R, I, P, T>
where
    R: OrderRepository,
    I: InventoryRepository,
    P: ProductRepository,
    T: CheckoutTransactionPort,
{
    pub fn new(order_repo: R, inventory_repo: I, product_repo: P, transaction_port: T) -> Self {
        Self {
            order_repo,
            inventory_repo,
            product_repo,
            transaction_port,
        }
    }

    pub async fn create_order(
        &self,
        input: CreateOrderInput,
    ) -> Result<Order, DomainError> {
        // 1. Idempotency check
        if let Some(existing) = self.order_repo
            .get_order_by_idempotency_key(&input.idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        // 2. Bulk fetch variants
        let variant_ids: Vec<Uuid> = input.items.iter().map(|i| i.variant_id).collect();
        let variants = self.product_repo
            .get_variants_by_ids(&variant_ids)
            .await?
            .into_iter()
            .map(|v| (v.id, v))
            .collect::<HashMap<_, _>>();

        if variants.len() != variant_ids.len() {
            return Err(DomainError::VariantNotFound);
        }

        // 3. Bulk fetch parent products (to get product_name)
        let product_ids: Vec<Uuid> = variants.values().map(|v| v.product_id).collect();
        let products = self.product_repo
            .get_products_by_ids(&product_ids)
            .await?
            .into_iter()
            .map(|p| (p.id, p))
            .collect::<HashMap<_, _>>();

        // 4. Bulk fetch stock snapshots
        let snapshots = self.inventory_repo
            .get_stock_snapshots_by_variant_ids(&variant_ids)
            .await?
            .into_iter()
            .map(|s| (s.variant_id, s))
            .collect::<HashMap<_, _>>();

        // 5. Prepare in‑memory structures
        let mut order_items = Vec::with_capacity(input.items.len());
        let mut ledger_entries = Vec::with_capacity(input.items.len());
        let mut updated_snapshots = Vec::with_capacity(input.items.len());
        let mut total_minor_units = 0;

        for item_input in input.items {
            let variant = variants.get(&item_input.variant_id)
                .ok_or(DomainError::VariantNotFound)?;
            let product = products.get(&variant.product_id)
                .ok_or(DomainError::ProductNotFound)?;
            let snapshot = snapshots.get(&item_input.variant_id)
                .ok_or(DomainError::SnapshotNotFound(item_input.variant_id))?;

            // Validate currency
            if variant.currency != input.currency {
                return Err(DomainError::CurrencyMismatch {
                    variant_id: variant.id,
                    variant_currency: variant.currency.clone(),
                    order_currency: input.currency.clone(),
                });
            }

            // Validate stock
            if snapshot.available_stock < item_input.quantity {
                return Err(DomainError::InsufficientStock {
                    variant_id: variant.id,
                    requested: item_input.quantity,
                    available: snapshot.available_stock,
                });
            }

            // Build OrderItem snapshot
            let order_item = OrderItem {
                id: Uuid::new_v4(),
                order_id: Uuid::nil(), // will be set after order creation
                variant_id: variant.id,
                quantity: item_input.quantity,
                product_name: product.name.clone(),   // from parent product
                sku: variant.sku.clone(),
                price_minor_units: variant.price_minor_units,
                currency: variant.currency.clone(),
                thumbnail_url: None,
                attributes: variant.attributes.clone(),
            };

            total_minor_units += variant.price_minor_units * item_input.quantity;
            order_items.push(order_item);

            // Prepare ledger entry (Reserve)
            let ledger_entry = InventoryLedgerEntry {
                id: Uuid::new_v4(),
                variant_id: variant.id,
                action: LedgerAction::Reserve,
                quantity_change: -item_input.quantity,
                reference_id: None, // will be set after order ID is generated
                idempotency_key: Some(format!("{}-{}", input.idempotency_key, variant.id)),
                created_at: Utc::now(),
            };
            ledger_entries.push(ledger_entry);

            // Prepare updated snapshot (with new version – will be incremented by DB)
            let mut updated_snapshot = snapshot.clone();
            updated_snapshot.available_stock -= item_input.quantity;
            updated_snapshot.reserved_stock += item_input.quantity;
            // version is left as is – the repository will increment it
            updated_snapshots.push(updated_snapshot);
        }

        // 6. Build Order aggregate
        let order_id = Uuid::new_v4();
        let now = Utc::now();
        let order = Order {
            id: order_id,
            user_id: input.user_id,
            state: OrderState::Pending,
            currency: input.currency,
            shipping_address: Some(input.shipping_address),
            billing_address: Some(input.billing_address),
            total_minor_units,
            version: 1,
            idempotency_key: input.idempotency_key,
            created_at: now,
            updated_at: now,
        };

        // Link order_items and ledger_entries to the order ID
        for item in &mut order_items {
            item.order_id = order_id;
        }
        for entry in &mut ledger_entries {
            entry.reference_id = Some(order_id);
        }

        // 7. Atomic commit
        self.transaction_port
            .commit_checkout_state(order, order_items, ledger_entries, updated_snapshots)
            .await?;

        // 8. Retrieve the persisted order (or we can return the in‑memory one)
        // We stored the order_id; we can fetch it again, or the port could return it.
        // For simplicity, we return the in‑memory order (the port will have saved it).
        // Better: return the order from the database after commit.
        let saved_order = self.order_repo.get_order_by_id(order_id).await?
            .ok_or(DomainError::OrderNotFound)?;
        Ok(saved_order)
    }
}