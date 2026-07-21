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

    async fn get_order_by_id(&self, _id: Uuid) -> Result<Option<Order>, DomainError> {
        unimplemented!()
    }

    async fn get_order_by_idempotency_key(&self, _key: &str) -> Result<Option<Order>, DomainError> {
        unimplemented!()
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
