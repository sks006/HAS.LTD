use crate::{ errors::DomainError, models::product::Variant };

pub trait InventoryRepository {
    fn get_variant(&self, variant_id: &str) -> Result<Variant, DomainError>;
    fn reserve_stock(&self, variant_id: &str, quantity: i32) -> Result<(), DomainError>;
    fn release_stock(&self, variant_id: &str, quantity: i32) -> Result<(), DomainError>;
}
