// backend/domain/src/models/product.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub category_id: Uuid,
    pub slug: String,
    pub name: String,
    pub fabric_type: String,
    pub season: Option<String>,
    pub description: Option<String>,
    pub images: Vec<String>, // Bounded array of CDN URIs
    pub featured_video_url: Option<String>,
    pub discount: Option<ProductTimeBoundDiscount>,
    pub aggregate_rating: Option<f32>, // Calculated via materialized view, not raw reviews
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductTimeBoundDiscount {
    pub percentage: u8,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductReview {
    pub id: Uuid,
    pub product_id: Uuid,
    pub rating: u8,
    pub user_id: Uuid, // MANDATORY: Sybil attack prevention
    pub is_verified_purchase: bool, // MANDATORY: Domain trust mappin
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>, // MANDATORY: Track state mutations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: Uuid,
    pub product_id: Uuid,
    pub sku: String,
    pub price_minor_units: i32, // Enforce integers for currency (e.g., Fils for AED)
    pub currency: String,
    pub is_active: bool,
    pub attributes: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
