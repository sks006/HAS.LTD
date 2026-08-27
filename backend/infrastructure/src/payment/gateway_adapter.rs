use async_trait::async_trait;
use domain::repositories::PaymentPort;
use domain::models::payment::PaymentIntent;
use domain::errors::DomainError;
use uuid::Uuid;
use chrono::Utc;

pub struct GatewayAdapter;

#[async_trait]
impl PaymentPort for GatewayAdapter {
    async fn create_intent(
        &self,
        order_id: Uuid,
        amount_minor_units: i32,
        currency: &str,
    ) -> Result<PaymentIntent, DomainError> {
        Ok(PaymentIntent {
            id: Uuid::new_v4(),
            order_id,
            amount_minor_units,
            currency: currency.to_string(),
            status: "requires_payment_method".to_string(),
            client_secret: format!("pi_mock_secret_{}", Uuid::new_v4()),
            created_at: Utc::now(),
        })
    }
}
