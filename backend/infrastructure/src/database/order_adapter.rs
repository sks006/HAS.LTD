use domain::{
    errors::DomainError,
    models::order::Order,
    repositories::order_port::OrderRepository,
};

pub struct OrderAdapter;

impl OrderRepository for OrderAdapter {
    fn save(&self, _order: &Order) -> Result<(), DomainError> {
        Ok(())
    }

    fn get_by_id(&self, order_id: &str) -> Result<Order, DomainError> {
        Ok(Order::new(order_id, "user-1", 4999))
    }
}
