// backend/infrastructure/src/database/product_repo.rs

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use domain::errors::DomainError;
use domain::models::product::{Product, ProductReview, ProductVariant};
use domain::repositories::{ProductFilters, ProductPagination, ProductRepository};

pub struct PgProductRepository {
    pub pool: PgPool,
}

impl PgProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProductRepository for PgProductRepository {
    async fn create_product(&self, _product: Product) -> Result<Product, DomainError> {
        unimplemented!()
    }

    async fn get_product_by_id(&self, _id: Uuid) -> Result<Option<Product>, DomainError> {
        unimplemented!()
    }

    async fn get_product_by_slug(&self, _slug: &str) -> Result<Option<Product>, DomainError> {
        unimplemented!()
    }

    async fn list_products(
        &self,
        _filters: ProductFilters,
        _pagination: ProductPagination,
    ) -> Result<Vec<Product>, DomainError> {
        unimplemented!()
    }

    async fn update_product(&self, _product: Product) -> Result<Product, DomainError> {
        unimplemented!()
    }

    async fn deactivate_product(&self, _id: Uuid) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn get_products_by_ids(&self, ids: &[Uuid]) -> Result<Vec<Product>, DomainError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT id, category_id, slug, name, fabric_type, season, description, images, featured_video_url, discount, aggregate_rating, is_active, created_at, updated_at
            FROM products
            WHERE id = ANY($1)
            "#
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut products = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            let discount_val: Option<serde_json::Value> = r.get("discount");
            let discount = match discount_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };

            products.push(Product {
                id: r.get("id"),
                category_id: r.get("category_id"),
                slug: r.get("slug"),
                name: r.get("name"),
                fabric_type: r.get("fabric_type"),
                season: r.get("season"),
                description: r.get("description"),
                images: r.get("images"),
                featured_video_url: r.get("featured_video_url"),
                discount,
                aggregate_rating: r.get("aggregate_rating"),
                is_active: r.get("is_active"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });
        }

        Ok(products)
    }

    async fn create_variant(&self, _variant: ProductVariant) -> Result<ProductVariant, DomainError> {
        unimplemented!()
    }

    async fn get_variant_by_id(&self, _id: Uuid) -> Result<Option<ProductVariant>, DomainError> {
        unimplemented!()
    }

    async fn get_variants_by_ids(&self, ids: &[Uuid]) -> Result<Vec<ProductVariant>, DomainError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            r#"
            SELECT id, product_id, sku, price_minor_units, currency, is_active, attributes, created_at, updated_at
            FROM product_variants
            WHERE id = ANY($1)
            "#
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut variants = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            variants.push(ProductVariant {
                id: r.get("id"),
                product_id: r.get("product_id"),
                sku: r.get("sku"),
                price_minor_units: r.get("price_minor_units"),
                currency: r.get("currency"),
                is_active: r.get("is_active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });
        }

        Ok(variants)
    }

    async fn list_variants_for_product(&self, _product_id: Uuid) -> Result<Vec<ProductVariant>, DomainError> {
        unimplemented!()
    }

    async fn update_variant(&self, _variant: ProductVariant) -> Result<ProductVariant, DomainError> {
        unimplemented!()
    }

    async fn deactivate_variant(&self, _id: Uuid) -> Result<(), DomainError> {
        unimplemented!()
    }

    async fn create_review(&self, _review: ProductReview) -> Result<ProductReview, DomainError> {
        unimplemented!()
    }

    async fn list_reviews_by_product(
        &self,
        _product_id: Uuid,
        _pagination: ProductPagination,
    ) -> Result<Vec<ProductReview>, DomainError> {
        unimplemented!()
    }
}
