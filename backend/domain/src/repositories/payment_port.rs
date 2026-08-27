use async_trait::async_trait;
use crate::models::payment::PaymentIntent;

#[async_trait]
pub trait PaymentPort: Send + Sync {
    async fn create_intent(
        &self,
        order_id: uuid::Uuid,
        amount_minor_units: i32,
        currency: &str,
    ) -> Result<PaymentIntent, crate::errors::DomainError>;
}
