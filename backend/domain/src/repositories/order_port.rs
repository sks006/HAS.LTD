use crate::{ errors::DomainError, models::order::Order };

pub trait OrderRepository {
    fn save(&self, order: &Order) -> Result<(), DomainError>;
    fn get_by_id(&self, order_id: &str) -> Result<Order, DomainError>;
}
