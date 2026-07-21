// backend/domain/src/errors.rs

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    InsufficientStock {
        variant_id: uuid::Uuid,
        requested: i32,
        available: i32,
    },
    ConcurrentStateModification, // Triggers on mismatched OCC version
    IdempotencyConflict,
    VariantNotFound,
    ProductNotFound,
    SnapshotNotFound(uuid::Uuid),
    OrderNotFound,
    UserNotFound,
    UserAlreadyExists,
    Unauthorized,
    InvalidStateTransition,
    CurrencyMismatch {
        variant_id: uuid::Uuid,
        variant_currency: String,
        order_currency: String,
    },
    DatabaseError(String),
    CacheError(String),
}