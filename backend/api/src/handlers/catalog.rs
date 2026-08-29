use axum::{ extract::{State, Path}, http::StatusCode, Json };
use serde::Serialize;
use uuid::Uuid;
use crate::state::AppState;
use crate::errors::ApiError;
use domain::repositories::{ProductRepository, ProductFilters, ProductPagination};
use infrastructure::database::product_repo::PgProductRepository;


#[derive(Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub price_cents: i64,
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

    let mut response = Vec::with_capacity(products.len());
    for p in products {
        let variants = product_repo.list_variants_for_product(p.id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let price_cents = variants.first().map(|v| v.price_minor_units as i64).unwrap_or(0);
        response.push(ProductResponse {
            id: p.id.to_string(),
            name: p.name,
            price_cents,
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
    let price_cents = variants.first().map(|v| v.price_minor_units as i64).unwrap_or(0);

    Ok(Json(ProductResponse {
        id: product.id.to_string(),
        name: product.name,
        price_cents,
    }))
}

