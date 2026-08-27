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

/// Retrieve details for a specific order, including its items.
pub async fn get_order(
    State(state): State<AppState>,
    RequireModeratorOrAdmin(_auth): RequireModeratorOrAdmin,
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
    RequireModeratorOrAdmin(_auth): RequireModeratorOrAdmin,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<UpdateOrderStatusDto>,
) -> Result<Json<Order>, ApiError> {
    payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

    let order_repo = PgOrderRepository::new(state.db_pool.clone());
    let new_state = parse_order_state(&payload.state)?;

    let updated_order = order_repo
        .update_order_state(order_id, payload.version, new_state)
        .await?;

    Ok(Json(updated_order))
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

