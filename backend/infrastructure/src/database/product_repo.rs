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

    async fn get_products_by_ids(&self, _ids: &[Uuid]) -> Result<Vec<Product>, DomainError> {
        unimplemented!()
    }

    async fn create_variant(&self, _variant: ProductVariant) -> Result<ProductVariant, DomainError> {
        unimplemented!()
    }

    async fn get_variant_by_id(&self, _id: Uuid) -> Result<Option<ProductVariant>, DomainError> {
        unimplemented!()
    }

    async fn get_variants_by_ids(&self, _ids: &[Uuid]) -> Result<Vec<ProductVariant>, DomainError> {
        unimplemented!()
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
