// backend/infrastructure/src/database/order_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::order::{Address, Order, OrderItem, OrderState};
use domain::repositories::OrderRepository;

pub struct PgOrderRepository {
    pub pool: PgPool,
}

impl PgOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepository for PgOrderRepository {
    async fn create_order(&self, _order: Order) -> Result<Order, DomainError> {
        unimplemented!()
    }

    async fn get_order_by_id(&self, id: Uuid) -> Result<Option<Order>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, state, currency, shipping_address, billing_address, total_minor_units, version, idempotency_key, created_at, updated_at
            FROM orders
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let shipping_val: Option<serde_json::Value> = r.get("shipping_address");
            let billing_val: Option<serde_json::Value> = r.get("billing_address");
            
            let shipping_address = match shipping_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };
            let billing_address = match billing_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };

            let state_str: String = r.get("state");
            let state = match state_str.as_str() {
                "Pending" => OrderState::Pending,
                "Reserved" => OrderState::Reserved,
                "Paid" => OrderState::Paid,
                "Shipped" => OrderState::Shipped,
                "Cancelled" => OrderState::Cancelled,
                _ => return Err(DomainError::Database(format!("Unknown order state: {}", state_str))),
            };

            Ok(Some(Order {
                id: r.get("id"),
                user_id: r.get("user_id"),
                state,
                currency: r.get("currency"),
                shipping_address,
                billing_address,
                total_minor_units: r.get("total_minor_units"),
                version: r.get("version"),
                idempotency_key: r.get("idempotency_key"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_order_by_idempotency_key(&self, key: &str) -> Result<Option<Order>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, state, currency, shipping_address, billing_address, total_minor_units, version, idempotency_key, created_at, updated_at
            FROM orders
            WHERE idempotency_key = $1
            "#
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let shipping_val: Option<serde_json::Value> = r.get("shipping_address");
            let billing_val: Option<serde_json::Value> = r.get("billing_address");
            
            let shipping_address = match shipping_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };
            let billing_address = match billing_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };

            let state_str: String = r.get("state");
            let state = match state_str.as_str() {
                "Pending" => OrderState::Pending,
                "Reserved" => OrderState::Reserved,
                "Paid" => OrderState::Paid,
                "Shipped" => OrderState::Shipped,
                "Cancelled" => OrderState::Cancelled,
                _ => return Err(DomainError::Database(format!("Unknown order state: {}", state_str))),
            };

            Ok(Some(Order {
                id: r.get("id"),
                user_id: r.get("user_id"),
                state,
                currency: r.get("currency"),
                shipping_address,
                billing_address,
                total_minor_units: r.get("total_minor_units"),
                version: r.get("version"),
                idempotency_key: r.get("idempotency_key"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_orders_by_user(
        &self,
        _user_id: Uuid,
        _limit: u32,
        _offset: u32,
    ) -> Result<Vec<Order>, DomainError> {
        unimplemented!()
    }

    async fn update_order_state(
        &self,
        _order_id: Uuid,
        _expected_version: i32,
        _new_state: OrderState,
    ) -> Result<Order, DomainError> {
        unimplemented!()
    }

    async fn set_shipping_address(
        &self,
        _order_id: Uuid,
        _address: Address,
    ) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn set_billing_address(
        &self,
        _order_id: Uuid,
        _address: Address,
    ) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn add_order_item(
        &self,
        _order_id: Uuid,
        _item: OrderItem,
    ) -> Result<OrderItem, DomainError> {
        unimplemented!()
    }

    async fn list_order_items(&self, _order_id: Uuid) -> Result<Vec<OrderItem>, DomainError> {
        unimplemented!()
    }
}
