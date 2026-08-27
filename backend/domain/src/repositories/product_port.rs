// backend/domain/src/repositories/product_port.rs

//! Defines the port (interface) for product‑related persistence.
//! The domain layer uses this trait to fetch/store products, variants, and reviews
//! without knowing anything about SQL or databases.

use async_trait::async_trait;   // Enables async methods in traits (Rust 1.75+)
use uuid::Uuid;                 // Universally unique identifiers for entity IDs

use crate::errors::DomainError; // Central error type for all domain failures
use crate::models::product::{Product, ProductVariant, ProductReview};

// ---------- Query DTOs ----------

/// Pagination parameters for list endpoints.
/// Exists to keep list methods clean and extendable.
pub struct ProductPagination {
    pub limit: u32,   // Max items per page (enforced by repository)
    pub offset: u32,  // Number of items to skip (for cursor‑based pagination later)
}

/// Filtering options for product listings.
/// Clients can filter by category, active status, or season.
/// All fields are optional; when `None`, no filter is applied.
pub struct ProductFilters {
    pub category_id: Option<Uuid>, // Limit to a specific category
    pub is_active: Option<bool>,   // If `Some(true)`, only show active products
    pub season: Option<String>,    // e.g., "Summer", "Winter"
}

// ---------- Repository Trait ----------

/// Main port for product aggregate persistence.
/// The `async_trait` macro allows `async fn` in traits.
/// `Send + Sync` ensures the repository can be shared across threads
/// (required by web frameworks like Axum/Actix).

#[async_trait]
pub trait ProductRepository: Send + Sync {
    // ---- Product ----

    /// Persist a new product. The implementation must:
    /// - Validate unique `slug`
    /// - Set `created_at` / `updated_at` if not already set
    /// - Return the full product (with generated ID and timestamps)
    async fn create_product(&self, product: Product) -> Result<Product, DomainError>;

    /// Fetch a product by its primary key. Returns `None` if not found.
    /// Used in detail pages, order creation, and admin panels.
    async fn get_product_by_id(&self, id: Uuid) -> Result<Option<Product>, DomainError>;

    /// Fetch a product by its URL‑friendly slug.
    /// This is the primary lookup for public product pages.
    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>, DomainError>;

    /// List products with optional filters and pagination.
    /// Used for category browsing, search results, and admin lists.
    async fn list_products(
        &self,
        filters: ProductFilters,
        pagination: ProductPagination,
    ) -> Result<Vec<Product>, DomainError>;

    /// Update an existing product (full replacement).
    /// The implementation must:
    /// - Enforce `updated_at` refresh
    /// - Check `version` if used (though Product currently has no `version` field)
    async fn update_product(&self, product: Product) -> Result<Product, DomainError>;

    /// Soft‑delete a product by setting `is_active = false`.
    /// This preserves historical order data and avoids cascade deletions.
    async fn deactivate_product(&self, id: Uuid) -> Result<(), DomainError>;

    /// Fetch multiple products by their IDs in bulk.
    async fn get_products_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>, DomainError>;

    // ---- Variants ----

    /// Fetch multiple variants by their IDs in bulk.
    async fn get_variants_by_ids(&self, ids: &[Uuid]) -> Result<Vec<ProductVariant>, DomainError>;

    /// Create a new product variant (SKU‑level).
    /// Must ensure `sku` uniqueness within the product.
    async fn create_variant(&self, variant: ProductVariant) -> Result<ProductVariant, DomainError>;

    /// Fetch a variant by its ID (used for order item snapshots and inventory).
    async fn get_variant_by_id(&self, id: Uuid) -> Result<Option<ProductVariant>, DomainError>;

    /// List all variants for a given product (e.g., for size/colour selectors).
    async fn list_variants_for_product(&self, product_id: Uuid) -> Result<Vec<ProductVariant>, DomainError>;

    /// Update a variant (price, attributes, currency, etc.).
    async fn update_variant(&self, variant: ProductVariant) -> Result<ProductVariant, DomainError>;

    /// Soft‑deactivate a variant (no longer available for sale).
    async fn deactivate_variant(&self, id: Uuid) -> Result<(), DomainError>;

    // ---- Reviews ----

    /// Create a review for a product. The implementation must:
    /// - Validate rating (1‑5)
    /// - Verify the user has purchased the product (via `is_verified_purchase`)
    /// - Prevent duplicate reviews from the same user
    async fn create_review(&self, review: ProductReview) -> Result<ProductReview, DomainError>;

    /// List reviews for a product with pagination (used on product detail pages).
    async fn list_reviews_by_product(
        &self,
        product_id: Uuid,
        pagination: ProductPagination,
    ) -> Result<Vec<ProductReview>, DomainError>;
}