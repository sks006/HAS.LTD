use axum::{ extract::State, http::StatusCode, Json };
use serde::Serialize;
use crate::state::AppState;
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
