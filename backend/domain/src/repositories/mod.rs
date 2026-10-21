//! Repository port module – exports all traits that the domain layer uses
//! to interact with persistence, cache, and transactions.

mod cache_port;
mod checkout_port;
mod inventory_port;
mod order_port;
mod product_port;
mod user_port;
mod storage_port;
mod hasher_port;
mod payment_port;
mod credentials_port;
mod session_port;
mod refresh_token_port;
mod mfa_port;
mod audit_log_port;

pub use cache_port::CachePort;
pub use checkout_port::CheckoutTransactionPort;
pub use inventory_port::InventoryRepository;
pub use order_port::OrderRepository;
pub use product_port::{ProductFilters, ProductPagination, ProductRepository};
pub use user_port::UserRepository;
pub use storage_port::StoragePort;
pub use hasher_port::HasherPort;
pub use payment_port::PaymentPort;
pub use credentials_port::CredentialsRepository;
pub use session_port::SessionRepository;
pub use refresh_token_port::RefreshTokenRepository;
pub use mfa_port::MfaRepository;
pub use audit_log_port::AuditLogRepository;