// backend/domain/src/repositories/order_port.rs
//! Defines the port for order persistence.
//! Orders are immutable after creation except for state transitions.
//! Optimistic concurrency control (`version`) prevents race conditions on state updates.
//! Idempotency keys ensure exactly‑once order creation.

use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::DomainError;
use crate::models::order::{Order, OrderItem, OrderState, Address};

#[async_trait]
pub trait OrderRepository: Send + Sync {
    // ---- Order ----

    /// Create a new order. The implementation must:
    /// - Check idempotency: if `idempotency_key` already exists, return the existing order (idempotent)
    /// - Generate a new `id` if not provided
    /// - Set `version = 1` for OCC
    /// - Persist both the order and its items in a single transaction
    async fn create_order(&self, order: Order) -> Result<Order, DomainError>;

    /// Fetch an order by its ID (used for order details, admin, and status checks).
    async fn get_order_by_id(&self, id: Uuid) -> Result<Option<Order>, DomainError>;

    /// Retrieve an order by its idempotency key.
    /// This allows clients to safely retry order creation without duplication.
    async fn get_order_by_idempotency_key(&self, key: &str) -> Result<Option<Order>, DomainError>;

    /// List all orders for a user, ordered by creation date descending.
    /// Used in the user's order history page.
    async fn list_orders_by_user(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Order>, DomainError>;

    async fn list_all_orders(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Order>, DomainError>;

    /// Delete an order and its items by ID.
    async fn delete_order(&self, order_id: Uuid) -> Result<(), DomainError>;

    /// Update the state of an order (e.g., Pending → Paid → Shipped).
    /// **OCC is mandatory**: the `expected_version` must equal the current `version`.
    /// If they match, the state is updated and `version` is incremented.
    /// If not, a `DomainError::ConcurrencyConflict` is returned.
    /// This prevents two admins from changing the same order simultaneously.
    async fn update_order_state(
        &self,
        order_id: Uuid,
        expected_version: i32,
        new_state: OrderState,
    ) -> Result<Order, DomainError>;

    /// Set the shipping address for an order (can be done after creation for split billing).
    /// The address is stored as a snapshot (immutable) – changes to the user’s profile
    /// do not affect historical orders.
    async fn set_shipping_address(
        &self,
        order_id: Uuid,
        address: Address,
    ) -> Result<(), DomainError>;

    /// Set the billing address (similar to shipping address).
    async fn set_billing_address(
        &self,
        order_id: Uuid,
        address: Address,
    ) -> Result<(), DomainError>;

    // ---- Order Items ----

    /// Add an item to an existing order (e.g., if the customer adds a product after checkout).
    /// The order must be in a modifiable state (`Pending` or `Reserved`).
    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: OrderItem,
    ) -> Result<OrderItem, DomainError>;

    /// Retrieve all items for an order (used for order detail display).
    async fn list_order_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, DomainError>;
}