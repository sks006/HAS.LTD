// backend/domain/src/models/order.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderState {
    Pending,
    Reserved,
    Paid,
    Shipped,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub state: OrderState,
    pub currency: String,
    pub shipping_address: Option<Address>,
    pub billing_address: Option<Address>,
    pub total_minor_units: i32,
    pub version: i32, // Mandatory for OCC
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub variant_id: Uuid,
    pub quantity: i32,
    pub product_name: String,   // e.g., "Duga Al Leil Sheila"
    pub sku: String,            // e.g., "W28-BLK"
    pub price_minor_units: i32,   // instead of price_at_purchase
    pub currency: String,
    pub thumbnail_url: Option<String>,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Address {
    pub recipient_name: String,
    pub phone: String,
    pub street_line1: String,
    pub street_line2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub postal_code: String,
    pub country: String,
     pub delivery_instructions: Option<String>,  // restored
}