use axum::{ extract::State, http::StatusCode, Json };
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub price_cents: i64,
}

pub async fn list_products(State(_state): State<AppState>) -> Result<
    Json<Vec<ProductResponse>>,
    StatusCode
> {
    let products = vec![ProductResponse {
        id: "product-1".to_string(),
        name: "Sample Product".to_string(),
        price_cents: 1999,
    }];

    Ok(Json(products))
}
