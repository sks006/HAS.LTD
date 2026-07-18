#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Paid,
    Shipped,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub status: OrderStatus,
    pub total_cents: i64,
}

impl Order {
    pub fn new(id: impl Into<String>, user_id: impl Into<String>, total_cents: i64) -> Self {
        Self {
            id: id.into(),
            user_id: user_id.into(),
            status: OrderStatus::Pending,
            total_cents,
        }
    }
}
