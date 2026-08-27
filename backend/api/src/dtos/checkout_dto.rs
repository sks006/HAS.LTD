use serde::{Deserialize, Serialize};
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddressDto {
    #[validate(length(min = 1, message = "Recipient name cannot be empty"))]
    pub recipient_name: String,
    
    #[validate(length(min = 1, message = "Phone cannot be empty"))]
    pub phone: String,
    
    #[validate(length(min = 1, message = "Street line 1 cannot be empty"))]
    pub street_line1: String,
    
    pub street_line2: Option<String>,
    
    #[validate(length(min = 1, message = "City cannot be empty"))]
    pub city: String,
    
    pub state: Option<String>,
    
    #[validate(length(min = 1, message = "Postal code cannot be empty"))]
    pub postal_code: String,
    
    #[validate(length(min = 1, message = "Country cannot be empty"))]
    pub country: String,
    
    pub delivery_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckoutItemDto {
    pub variant_id: Uuid,
    
    #[validate(range(min = 1, message = "Quantity must be at least 1"))]
    pub quantity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CheckoutRequestDto {
    #[validate(length(min = 3, max = 3, message = "Currency must be a 3-letter ISO code"))]
    pub currency: String,
    
    #[validate(nested)]
    pub shipping_address: AddressDto,
    
    #[validate(nested)]
    pub billing_address: Option<AddressDto>,
    
    #[validate(custom(function = "validate_items"))]
    pub items: Vec<CheckoutItemDto>,
}

fn validate_items(items: &[CheckoutItemDto]) -> Result<(), validator::ValidationError> {
    if items.is_empty() {
        return Err(validator::ValidationError::new("items_empty"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateOrderStatusDto {
    #[validate(length(min = 1, message = "State cannot be empty"))]
    pub state: String,
    pub version: i32,
}
