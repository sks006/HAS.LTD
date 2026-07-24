// backend/infrastructure/src/database/order_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::order::{Address, Order, OrderItem, OrderState};
use domain::repositories::OrderRepository;

fn order_state_to_str(state: &OrderState) -> &'static str {
    match state {
        OrderState::Pending => "Pending",
        OrderState::Reserved => "Reserved",
        OrderState::Paid => "Paid",
        OrderState::Shipped => "Shipped",
        OrderState::Cancelled => "Cancelled",
    }
}

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
    async fn create_order(&self, order: Order) -> Result<Order, DomainError> {
        if let Some(existing) = self.get_order_by_idempotency_key(&order.idempotency_key).await? {
            return Ok(existing);
        }

        let shipping_json = serde_json::to_value(&order.shipping_address)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;
        let billing_json = serde_json::to_value(&order.billing_address)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO orders (
                id, user_id, state, currency,
                shipping_address, billing_address,
                total_minor_units, version, idempotency_key,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(order.id)
        .bind(order.user_id)
        .bind(order_state_to_str(&order.state))
        .bind(&order.currency)
        .bind(shipping_json)
        .bind(billing_json)
        .bind(order.total_minor_units)
        .bind(order.version)
        .bind(&order.idempotency_key)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    if db_err.message().contains("idempotency_key") {
                        return DomainError::IdempotencyCollision(order.idempotency_key.clone());
                    }
                }
            }
            DomainError::Database(e.to_string())
        })?;

        Ok(order)
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
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Order>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, state, currency, shipping_address, billing_address, total_minor_units, version, idempotency_key, created_at, updated_at
            FROM orders
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut orders = Vec::with_capacity(rows.len());
        for r in rows {
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

            orders.push(Order {
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
            });
        }
        Ok(orders)
    }

    async fn update_order_state(
        &self,
        order_id: Uuid,
        expected_version: i32,
        new_state: OrderState,
    ) -> Result<Order, DomainError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE orders
            SET state = $1, version = version + 1, updated_at = NOW()
            WHERE id = $2 AND version = $3
            "#
        )
        .bind(order_state_to_str(&new_state))
        .bind(order_id)
        .bind(expected_version)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            if let Some(existing) = self.get_order_by_id(order_id).await? {
                return Err(DomainError::ConcurrencyConflict {
                    message: format!(
                        "Order {} version conflict: expected version {}, found version {}",
                        order_id, expected_version, existing.version
                    ),
                });
            } else {
                return Err(DomainError::OrderNotFound);
            }
        }

        self.get_order_by_id(order_id)
            .await?
            .ok_or(DomainError::OrderNotFound)
    }

    async fn set_shipping_address(
        &self,
        order_id: Uuid,
        address: Address,
    ) -> Result<(), DomainError> {
        let address_json = serde_json::to_value(&address)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE orders
            SET shipping_address = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(address_json)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::OrderNotFound);
        }
        Ok(())
    }

    async fn set_billing_address(
        &self,
        order_id: Uuid,
        address: Address,
    ) -> Result<(), DomainError> {
        let address_json = serde_json::to_value(&address)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE orders
            SET billing_address = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(address_json)
        .bind(order_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::OrderNotFound);
        }
        Ok(())
    }

    async fn add_order_item(
        &self,
        order_id: Uuid,
        item: OrderItem,
    ) -> Result<OrderItem, DomainError> {
        let mut tx = self.pool.begin().await.map_err(|e| DomainError::Database(e.to_string()))?;

        let order = sqlx::query(
            r#"
            SELECT state FROM orders WHERE id = $1 FOR UPDATE
            "#
        )
        .bind(order_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .ok_or(DomainError::OrderNotFound)?;

        use sqlx::Row;
        let state_str: String = order.get("state");
        if state_str != "Pending" && state_str != "Reserved" {
            return Err(DomainError::InvalidStateTransition);
        }

        let attrs = serde_json::to_value(&item.attributes)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO order_items (id, order_id, variant_id, quantity, product_name, sku, price_minor_units, currency, thumbnail_url, attributes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(item.id)
        .bind(order_id)
        .bind(item.variant_id)
        .bind(item.quantity)
        .bind(&item.product_name)
        .bind(&item.sku)
        .bind(item.price_minor_units)
        .bind(&item.currency)
        .bind(&item.thumbnail_url)
        .bind(attrs)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let item_price_total = item.price_minor_units * item.quantity;
        sqlx::query(
            r#"
            UPDATE orders
            SET total_minor_units = total_minor_units + $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(item_price_total)
        .bind(order_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        tx.commit().await.map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(item)
    }

    async fn list_order_items(&self, order_id: Uuid) -> Result<Vec<OrderItem>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, order_id, variant_id, quantity, product_name, sku, price_minor_units, currency, thumbnail_url, attributes
            FROM order_items
            WHERE order_id = $1
            "#
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut items = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            items.push(OrderItem {
                id: r.get("id"),
                order_id: r.get("order_id"),
                variant_id: r.get("variant_id"),
                quantity: r.get("quantity"),
                product_name: r.get("product_name"),
                sku: r.get("sku"),
                price_minor_units: r.get("price_minor_units"),
                currency: r.get("currency"),
                thumbnail_url: r.get("thumbnail_url"),
                attributes: r.get("attributes"),
            });
        }
        Ok(items)
    }
}
