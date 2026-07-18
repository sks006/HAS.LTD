use domain::{
    errors::DomainError,
    models::product::Variant,
    repositories::inventory_port::InventoryRepository,
};

pub struct InventoryAdapter;

impl InventoryRepository for InventoryAdapter {
    fn get_variant(&self, variant_id: &str) -> Result<Variant, DomainError> {
        Ok(Variant {
            id: variant_id.to_string(),
            product_id: "product-1".to_string(),
            sku: format!("sku-{variant_id}"),
            stock: 100,
            price_cents: 1999,
        })
    }

    fn reserve_stock(&self, _variant_id: &str, _quantity: i32) -> Result<(), DomainError> {
        Ok(())
    }

    fn release_stock(&self, _variant_id: &str, _quantity: i32) -> Result<(), DomainError> {
        Ok(())
    }
}
