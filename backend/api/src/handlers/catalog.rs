use axum::{extract::{State, Path}, http::StatusCode, Json};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;
use crate::state::AppState;
use crate::errors::ApiError;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use domain::repositories::{ProductRepository, ProductFilters, ProductPagination};
use infrastructure::database::product_repo::PgProductRepository;

#[derive(Serialize)]
pub struct ProductResponse {
    pub id: String,
    pub name: String,
    pub price_cents: i64,
    pub price_formatted: String,
    pub category: String,
    pub images: Vec<String>,
    pub stock: i32,
    pub incoming: i32,
}

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub price: Decimal,
    pub category_id: Option<Uuid>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub fabric_type: Option<String>,
    pub season: Option<String>,
    pub images: Option<Vec<String>>,
    pub stock: Option<i32>,
    pub incoming: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub price: Option<Decimal>,
    pub category_id: Option<Uuid>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub fabric_type: Option<String>,
    pub season: Option<String>,
    pub images: Option<Vec<String>>,
    pub stock: Option<i32>,
    pub incoming: Option<i32>,
}

fn parse_stock_incoming(desc: Option<&str>, default_stock: i32, default_incoming: i32) -> (i32, i32) {
    if let Some(d) = desc {
        let mut stock = default_stock;
        let mut incoming = default_incoming;
        for part in d.split('|') {
            if let Some(val) = part.strip_prefix("Stock:") {
                if let Ok(s) = val.parse::<i32>() { stock = s; }
            } else if let Some(val) = part.strip_prefix("Incoming:") {
                if let Ok(inc) = val.parse::<i32>() { incoming = inc; }
            }
        }
        return (stock, incoming);
    }
    (default_stock, default_incoming)
}

pub async fn create_product(
    State(state): State<AppState>,
    Json(payload): Json<CreateProductRequest>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let id = Uuid::new_v4();
    let category_id = payload.category_id.unwrap_or_else(Uuid::new_v4);
    let slug = payload.slug.unwrap_or_else(|| payload.name.to_lowercase().replace(' ', "-"));
    let fabric_type = payload.fabric_type.unwrap_or_else(|| "Velvet & Silk".to_string());
    let now = chrono::Utc::now();

    let stock_val = payload.stock.unwrap_or(100);
    let incoming_val = payload.incoming.unwrap_or(50);
    let description = payload.description.or_else(|| {
        Some(format!("Stock:{}|Incoming:{}", stock_val, incoming_val))
    });

    let product = domain::models::product::Product {
        id,
        category_id,
        slug,
        name: payload.name.clone(),
        price: payload.price,
        fabric_type,
        season: payload.season,
        description,
        images: payload.images.unwrap_or_default(),
        featured_video_url: None,
        discount: None,
        aggregate_rating: Some(0.0),
        is_active: true,
        created_at: now,
        updated_at: now,
    };

    let created = product_repo.create_product(product).await?;

    let price_cents = (created.price * Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0);

    let (stock, incoming) = parse_stock_incoming(created.description.as_deref(), stock_val, incoming_val);

    Ok(Json(ProductResponse {
        id: created.id.to_string(),
        name: created.name.clone(),
        price_cents,
        price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
        category: created.fabric_type.clone(),
        images: created.images.clone(),
        stock,
        incoming,
    }))
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

        let price_cents = if let Some(v) = first_variant {
            v.price_minor_units as i64
        } else {
            (p.price * Decimal::from(100))
                .round()
                .to_i64()
                .unwrap_or(0)
        };

        let (stock, incoming) = parse_stock_incoming(p.description.as_deref(), 100, 50);

        response.push(ProductResponse {
            id: p.id.to_string(),
            name: p.name,
            price_cents,
            price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
            category: p.fabric_type,
            images: p.images,
            stock,
            incoming,
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

    let (stock, incoming) = parse_stock_incoming(product.description.as_deref(), 100, 50);

    Ok(Json(ProductResponse {
        id: product.id.to_string(),
        name: product.name,
        price_cents,
        price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
        category: product.fabric_type,
        images: product.images,
        stock,
        incoming,
    }))
}

pub async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    Json(payload): Json<UpdateProductRequest>,
) -> Result<Json<ProductResponse>, ApiError> {
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let mut product = product_repo.get_product_by_id(product_id).await?
        .ok_or_else(|| ApiError::NotFound("Product not found".to_string()))?;

    if let Some(name) = payload.name { product.name = name; }
    if let Some(price) = payload.price { product.price = price; }
    if let Some(cat_id) = payload.category_id { product.category_id = cat_id; }
    if let Some(slug) = payload.slug { product.slug = slug; }
    if let Some(fabric) = payload.fabric_type { product.fabric_type = fabric; }
    if let Some(season) = payload.season { product.season = Some(season); }
    if let Some(images) = payload.images { product.images = images; }

    if let Some(stk) = payload.stock {
        let inc = payload.incoming.unwrap_or(50);
        product.description = Some(format!("Stock:{}|Incoming:{}", stk, inc));
    } else if let Some(desc) = payload.description {
        product.description = Some(desc);
    }
    
    product.updated_at = chrono::Utc::now();

    let updated = product_repo.update_product(product).await?;

    let price_cents = (updated.price * Decimal::from(100))
        .round()
        .to_i64()
        .unwrap_or(0);

    let (stock, incoming) = parse_stock_incoming(updated.description.as_deref(), 100, 50);

    Ok(Json(ProductResponse {
        id: updated.id.to_string(),
        name: updated.name,
        price_cents,
        price_formatted: format!("${:.2}", price_cents as f64 / 100.0),
        category: updated.fabric_type,
        images: updated.images,
        stock,
        incoming,
    }))
}

pub async fn delete_product(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let _ = product_repo.deactivate_product(product_id).await;

    let _ = sqlx::query("DELETE FROM product_variants WHERE product_id = $1")
        .bind(product_id)
        .execute(&state.db_pool)
        .await;

    let _ = sqlx::query("DELETE FROM product_reviews WHERE product_id = $1")
        .bind(product_id)
        .execute(&state.db_pool)
        .await;

    let _ = sqlx::query("DELETE FROM inventory_ledger WHERE product_id = $1")
        .bind(product_id)
        .execute(&state.db_pool)
        .await;

    let res = sqlx::query("DELETE FROM products WHERE id = $1")
        .bind(product_id)
        .execute(&state.db_pool)
        .await;

    match res {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            eprintln!("Database delete error: {:?}", e);
            Err(ApiError::Internal("Failed to delete product from database".to_string()))
        }
    }
}
