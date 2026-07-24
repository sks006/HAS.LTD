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
    async fn create_product(&self, product: Product) -> Result<Product, DomainError> {
        let discount_json = serde_json::to_value(&product.discount)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO products (id, category_id, slug, name, fabric_type, season, description, images, featured_video_url, discount, aggregate_rating, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(product.id)
        .bind(product.category_id)
        .bind(&product.slug)
        .bind(&product.name)
        .bind(&product.fabric_type)
        .bind(&product.season)
        .bind(&product.description)
        .bind(&product.images)
        .bind(&product.featured_video_url)
        .bind(discount_json)
        .bind(product.aggregate_rating)
        .bind(product.is_active)
        .bind(product.created_at)
        .bind(product.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(product)
    }

    async fn get_product_by_id(&self, id: Uuid) -> Result<Option<Product>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, category_id, slug, name, fabric_type, season, description, images, featured_video_url, discount, aggregate_rating, is_active, created_at, updated_at
            FROM products
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let discount_val: Option<serde_json::Value> = r.get("discount");
            let discount = match discount_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };

            Ok(Some(Product {
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
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, category_id, slug, name, fabric_type, season, description, images, featured_video_url, discount, aggregate_rating, is_active, created_at, updated_at
            FROM products
            WHERE slug = $1
            "#
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            let discount_val: Option<serde_json::Value> = r.get("discount");
            let discount = match discount_val {
                Some(v) => serde_json::from_value(v).map_err(|e| DomainError::Serialization(e.to_string()))?,
                None => None,
            };

            Ok(Some(Product {
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
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_products(
        &self,
        filters: ProductFilters,
        pagination: ProductPagination,
    ) -> Result<Vec<Product>, DomainError> {
        let mut query_builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "SELECT id, category_id, slug, name, fabric_type, season, description, images, featured_video_url, discount, aggregate_rating, is_active, created_at, updated_at FROM products WHERE 1=1"
        );

        if let Some(cat_id) = filters.category_id {
            query_builder.push(" AND category_id = ").push_bind(cat_id);
        }
        if let Some(active) = filters.is_active {
            query_builder.push(" AND is_active = ").push_bind(active);
        }
        if let Some(season) = filters.season {
            query_builder.push(" AND season = ").push_bind(season);
        }

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(pagination.limit as i64);
        query_builder.push(" OFFSET ").push_bind(pagination.offset as i64);

        let rows = query_builder.build()
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

    async fn update_product(&self, product: Product) -> Result<Product, DomainError> {
        let discount_json = serde_json::to_value(&product.discount)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        let rows_affected = sqlx::query(
            r#"
            UPDATE products
            SET category_id = $1, slug = $2, name = $3, fabric_type = $4, season = $5, description = $6, images = $7, featured_video_url = $8, discount = $9, aggregate_rating = $10, is_active = $11, updated_at = NOW()
            WHERE id = $12
            "#
        )
        .bind(product.category_id)
        .bind(&product.slug)
        .bind(&product.name)
        .bind(&product.fabric_type)
        .bind(&product.season)
        .bind(&product.description)
        .bind(&product.images)
        .bind(&product.featured_video_url)
        .bind(discount_json)
        .bind(product.aggregate_rating)
        .bind(product.is_active)
        .bind(product.id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::ProductNotFound);
        }

        Ok(product)
    }

    async fn deactivate_product(&self, id: Uuid) -> Result<(), DomainError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE products
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::ProductNotFound);
        }
        Ok(())
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

    async fn create_variant(&self, variant: ProductVariant) -> Result<ProductVariant, DomainError> {
        sqlx::query(
            r#"
            INSERT INTO product_variants (id, product_id, sku, price_minor_units, currency, is_active, attributes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(variant.id)
        .bind(variant.product_id)
        .bind(&variant.sku)
        .bind(variant.price_minor_units)
        .bind(&variant.currency)
        .bind(variant.is_active)
        .bind(&variant.attributes)
        .bind(variant.created_at)
        .bind(variant.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(variant)
    }

    async fn get_variant_by_id(&self, id: Uuid) -> Result<Option<ProductVariant>, DomainError> {
        let row = sqlx::query(
            r#"
            SELECT id, product_id, sku, price_minor_units, currency, is_active, attributes, created_at, updated_at
            FROM product_variants
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if let Some(r) = row {
            use sqlx::Row;
            Ok(Some(ProductVariant {
                id: r.get("id"),
                product_id: r.get("product_id"),
                sku: r.get("sku"),
                price_minor_units: r.get("price_minor_units"),
                currency: r.get("currency"),
                is_active: r.get("is_active"),
                attributes: r.get("attributes"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
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

    async fn list_variants_for_product(&self, product_id: Uuid) -> Result<Vec<ProductVariant>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, product_id, sku, price_minor_units, currency, is_active, attributes, created_at, updated_at
            FROM product_variants
            WHERE product_id = $1
            "#
        )
        .bind(product_id)
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

    async fn update_variant(&self, variant: ProductVariant) -> Result<ProductVariant, DomainError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE product_variants
            SET product_id = $1, sku = $2, price_minor_units = $3, currency = $4, is_active = $5, attributes = $6, updated_at = NOW()
            WHERE id = $7
            "#
        )
        .bind(variant.product_id)
        .bind(&variant.sku)
        .bind(variant.price_minor_units)
        .bind(&variant.currency)
        .bind(variant.is_active)
        .bind(&variant.attributes)
        .bind(variant.id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::VariantNotFound);
        }

        Ok(variant)
    }

    async fn deactivate_variant(&self, id: Uuid) -> Result<(), DomainError> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE product_variants
            SET is_active = false, updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .rows_affected();

        if rows_affected == 0 {
            return Err(DomainError::VariantNotFound);
        }
        Ok(())
    }

    async fn create_review(&self, review: ProductReview) -> Result<ProductReview, DomainError> {
        if review.rating < 1 || review.rating > 5 {
            return Err(DomainError::Database("Rating must be between 1 and 5".to_string()));
        }

        let duplicate = sqlx::query(
            r#"
            SELECT 1 FROM product_reviews
            WHERE product_id = $1 AND user_id = $2
            LIMIT 1
            "#
        )
        .bind(review.product_id)
        .bind(review.user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if duplicate.is_some() {
            return Err(DomainError::Database("User has already reviewed this product".to_string()));
        }

        let purchased = sqlx::query(
            r#"
            SELECT 1 FROM orders o
            JOIN order_items oi ON o.id = oi.order_id
            JOIN product_variants pv ON oi.variant_id = pv.id
            WHERE o.user_id = $1 AND pv.product_id = $2 AND o.state IN ('Paid', 'Shipped')
            LIMIT 1
            "#
        )
        .bind(review.user_id)
        .bind(review.product_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?
        .is_some();

        sqlx::query(
            r#"
            INSERT INTO product_reviews (id, product_id, rating, user_id, is_verified_purchase, comment, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(review.id)
        .bind(review.product_id)
        .bind(review.rating as i16)
        .bind(review.user_id)
        .bind(purchased)
        .bind(&review.comment)
        .bind(review.created_at)
        .bind(review.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut final_review = review;
        final_review.is_verified_purchase = purchased;
        Ok(final_review)
    }

    async fn list_reviews_by_product(
        &self,
        product_id: Uuid,
        pagination: ProductPagination,
    ) -> Result<Vec<ProductReview>, DomainError> {
        let rows = sqlx::query(
            r#"
            SELECT id, product_id, rating, user_id, is_verified_purchase, comment, created_at, updated_at
            FROM product_reviews
            WHERE product_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(product_id)
        .bind(pagination.limit as i64)
        .bind(pagination.offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let mut reviews = Vec::with_capacity(rows.len());
        for r in rows {
            use sqlx::Row;
            let rating_i16: i16 = r.get("rating");
            reviews.push(ProductReview {
                id: r.get("id"),
                product_id: r.get("product_id"),
                rating: rating_i16 as u8,
                user_id: r.get("user_id"),
                is_verified_purchase: r.get("is_verified_purchase"),
                comment: r.get("comment"),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });
        }
        Ok(reviews)
    }
}
