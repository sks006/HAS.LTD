// backend/infrastructure/src/email/mod.rs

pub mod in_memory_adapter;
pub mod resend_adapter;

pub use in_memory_adapter::InMemoryEmailAdapter;
pub use resend_adapter::ResendEmailAdapter;
