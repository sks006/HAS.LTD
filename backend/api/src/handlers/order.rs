use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;
use serde::Serialize;

use crate::{
    dtos::UpdateOrderStatusDto,
    errors::ApiError,
    middleware::RequireModeratorOrAdmin,
    state::AppState,
};

use domain::repositories::OrderRepository;
use domain::models::order::{Order, OrderItem, OrderState};
use infrastructure::database::order_repo::PgOrderRepository;

#[derive(Serialize)]
pub struct OrderDetailsResponse {
    pub order: Order,
    pub items: Vec<OrderItem>,
}

fn parse_order_state(s: &str) -> Result<OrderState, ApiError> {
    match s {
        "Pending" => Ok(OrderState::Pending),
        "Reserved" => Ok(OrderState::Reserved),
        "Paid" => Ok(OrderState::Paid),
        "Shipped" => Ok(OrderState::Shipped),
        "Cancelled" => Ok(OrderState::Cancelled),
        _ => Err(ApiError::BadRequest(format!("Invalid order state: {}", s))),
    }
}

#[derive(Serialize)]
pub struct OrderRecordDto {
    pub id: Uuid,
    pub order_number: String,
    pub customer_name: String,
    pub product_name: String,
    pub sku: String,
    pub payment_method: String,
    pub amount: f64,
    pub currency: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub version: i32,
    pub idempotency_key: String,
    pub shipping_address: Option<domain::models::order::Address>,
}

/// Retrieve all orders for dashboard/admin listing.
pub async fn list_orders(
    State(state): State<AppState>,
) -> Result<Json<Vec<OrderRecordDto>>, ApiError> {
    let order_repo = PgOrderRepository::new(state.db_pool.clone());
    let raw_orders = order_repo.list_all_orders(100, 0).await?;

    let mut records = Vec::with_capacity(raw_orders.len());
    for o in raw_orders {
        let items = order_repo.list_order_items(o.id).await.unwrap_or_default();
        let main_item = items.first();
        let customer_name = o.shipping_address
            .as_ref()
            .map(|a| a.recipient_name.clone())
            .unwrap_or_else(|| "Customer".to_string());
        
        let product_name = main_item
            .map(|i| i.product_name.clone())
            .unwrap_or_else(|| "Haute Couture Abaya".to_string());

        let sku = main_item
            .map(|i| i.sku.clone())
            .unwrap_or_else(|| "SK-45".to_string());

        let item_sum_minor: i32 = items.iter().map(|i| i.price_minor_units * i.quantity).sum();
        let amount = if o.total_minor_units > 0 {
            o.total_minor_units as f64 / 100.0
        } else if item_sum_minor > 0 {
            item_sum_minor as f64 / 100.0
        } else {
            0.0
        };

        records.push(OrderRecordDto {
            id: o.id,
            order_number: format!("ID: {}", &o.id.to_string()[..8]),
            customer_name,
            product_name,
            sku,
            payment_method: "Cash on Delivery".to_string(),
            amount,
            currency: o.currency.clone(),
            status: format!("{:?}", o.state),
            created_at: o.created_at.to_rfc3339(),
            updated_at: Some(o.updated_at.to_rfc3339()),
            version: o.version,
            idempotency_key: o.idempotency_key,
            shipping_address: o.shipping_address,
        });
    }

    Ok(Json(records))
}

/// Retrieve details for a specific order, including its items.
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<OrderDetailsResponse>, ApiError> {
    let order_repo = PgOrderRepository::new(state.db_pool.clone());

    let order = order_repo.get_order_by_id(order_id).await?
        .ok_or_else(|| ApiError::NotFound("Order not found".to_string()))?;

    let items = order_repo.list_order_items(order_id).await?;

    Ok(Json(OrderDetailsResponse { order, items }))
}

/// Update the status/state of an order with optimistic concurrency control (version).
pub async fn update_order_status(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusDto>,
) -> Result<Json<Order>, ApiError> {
    let order_repo = PgOrderRepository::new(state.db_pool.clone());
    let new_state = parse_order_state(&payload.state)?;

    let updated_order = order_repo
        .update_order_state(order_id, payload.version, new_state)
        .await?;

    Ok(Json(updated_order))
}

/// Delete an order and its items by ID.
pub async fn delete_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let order_repo = PgOrderRepository::new(state.db_pool.clone());
    order_repo.delete_order(order_id).await?;
    Ok(Json(serde_json::json!({ "ok": true, "message": "Order deleted successfully" })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_order_state_success() {
        assert_eq!(parse_order_state("Pending").unwrap(), OrderState::Pending);
        assert_eq!(parse_order_state("Reserved").unwrap(), OrderState::Reserved);
        assert_eq!(parse_order_state("Paid").unwrap(), OrderState::Paid);
        assert_eq!(parse_order_state("Shipped").unwrap(), OrderState::Shipped);
        assert_eq!(parse_order_state("Cancelled").unwrap(), OrderState::Cancelled);
    }

    #[test]
    fn test_parse_order_state_failure() {
        let err = parse_order_state("UnknownState");
        assert!(err.is_err());
        match err.unwrap_err() {
            ApiError::BadRequest(msg) => {
                assert_eq!(msg, "Invalid order state: UnknownState");
            }
            _ => panic!("Expected ApiError::BadRequest"),
        }
    }
}

