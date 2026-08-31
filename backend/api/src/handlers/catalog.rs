use axum::{ extract::{State, Path}, http::StatusCode, Json };
use serde::Serialize;
use uuid::Uuid;
use std::collections::HashMap;
use crate::state::AppState;
use crate::errors::ApiError;
use rust_decimal::{Decimal, prelude::ToPrimitive};          // <-- import Decimal
use domain::repositories::{ProductRepository, ProductFilters, ProductPagination};
use infrastructure::database::product_repo::PgProductRepository;


#[derive(Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub price_cents: i64,
    pub price_formatted: String,    //for display only
}

pub async fn list_products(State(state): State<AppState>) -> Result<
    Json<Vec<ProductResponse>>,
    StatusCode
> {
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let filters = ProductFilters {
        category_id: None,
        is_active: Some(true),
        season: None,
    };
    let pagination = ProductPagination {
        limit: 100,
        offset: 0,
    };

    let products = product_repo.list_products(filters, pagination).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let product_ids: Vec<Uuid> = products.iter().map(|p| p.id).collect();
    let all_variants = product_repo.get_variants_by_product_ids(&product_ids).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let variants_by_product: HashMap<Uuid, Vec<domain::models::product::ProductVariant>> = 
        all_variants.into_iter().fold(HashMap::new(), |mut acc, v| {
            acc.entry(v.product_id).or_default().push(v);
            acc
        });

    let mut response = Vec::with_capacity(products.len());
    for p in products {
        let first_variant = variants_by_product.get(&p.id).and_then(|v| v.first());

        // Determine price in cents
        let price_cents = if let Some(v) = first_variant {
            // Variant price (already in cents)
            v.price_minor_units as i64
        } else {
            // Fallback to product’s own price (convert Decimal → cents)
            (p.price * Decimal::from(100))
                .round()
                .to_i64()
                .unwrap_or(0)
        };

        response.push(ProductResponse {
            id: p.id.to_string(),
            name: p.name,
            price_cents,
            price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
        });
    }

    Ok(Json(response))
}

pub async fn get_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let product = product_repo.get_product_by_id(product_id).await?
        .ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;

    let variants = product_repo.list_variants_for_product(product.id).await?;
       let price_cents = if let Some(v) = variants.first() {
        v.price_minor_units as i64
    } else {
        (product.price * Decimal::from(100))
            .round()
            .to_i64()
            .unwrap_or(0)
    };


    Ok(Json(ProductResponse {
        id: product.id.to_string(),
        name: product.name,
        price_cents,
        price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
    }))
}

