use axum::{ extract::State, Json };
use serde::Serialize;
use validator::Validate;

use crate::{
    dtos::CheckoutRequestDto,
    errors::ApiError,
    extractors::{ AuthExtractor, IdempotencyExtractor },
    state::AppState,
};

use infrastructure::database::{
    order_repo::PgOrderRepository,
    inventory_repo::SqlxInventoryRepository,
    product_repo::PgProductRepository,
    checkout_adapter::SqlxCheckoutAdapter,
};
use domain::services::checkout::{CheckoutService, CreateOrderInput, OrderItemInput};

#[derive(Serialize)]
pub struct CheckoutResponse {
    pub ok: bool,
    pub message: String,
}

pub async fn checkout(
    State(state): State<AppState>,
    AuthExtractor(auth): AuthExtractor,
    IdempotencyExtractor(idempotency_key): IdempotencyExtractor,
    Json(payload): Json<CheckoutRequestDto>,
) -> Result<Json<CheckoutResponse>, ApiError> {
    // 1. Validate payload
    payload.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

    // 2. Instantiate repository adapters
    let order_repo = PgOrderRepository::new(state.db_pool.clone());
    let inventory_repo = SqlxInventoryRepository::new(state.db_pool.clone());
    let product_repo = PgProductRepository::new(state.db_pool.clone());
    let transaction_port = SqlxCheckoutAdapter::new(state.db_pool.clone());

    // 3. Create domain service
    let checkout_service = CheckoutService::new(
        order_repo,
        inventory_repo,
        product_repo,
        transaction_port,
    );

    // 4. Map DTO to domain input types
    let items = payload.items.into_iter().map(|item| OrderItemInput {
        variant_id: item.variant_id,
        quantity: item.quantity,
    }).collect();

    let shipping_address = domain::models::order::Address {
        recipient_name: payload.shipping_address.recipient_name,
        phone: payload.shipping_address.phone,
        street_line1: payload.shipping_address.street_line1,
        street_line2: payload.shipping_address.street_line2,
        city: payload.shipping_address.city,
        state: payload.shipping_address.state,
        postal_code: payload.shipping_address.postal_code,
        country: payload.shipping_address.country,
        delivery_instructions: payload.shipping_address.delivery_instructions,
    };

    let billing_address = match payload.billing_address {
        Some(addr) => domain::models::order::Address {
            recipient_name: addr.recipient_name,
            phone: addr.phone,
            street_line1: addr.street_line1,
            street_line2: addr.street_line2,
            city: addr.city,
            state: addr.state,
            postal_code: addr.postal_code,
            country: addr.country,
            delivery_instructions: addr.delivery_instructions,
        },
        None => shipping_address.clone(),
    };

    let input = CreateOrderInput {
        user_id: Some(auth.sub),
        items,
        shipping_address,
        billing_address,
        idempotency_key,
        currency: payload.currency,
    };

    // 5. Execute checkout orchestrator
    let order = checkout_service.create_order(input).await?;

    Ok(
        Json(CheckoutResponse {
            ok: true,
            message: format!("checkout accepted: order {}", order.id),
        })
    )
}
