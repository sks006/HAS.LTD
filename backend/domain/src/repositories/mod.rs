//! Repository port module – exports all traits that the domain layer uses
//! to interact with persistence, cache, and transactions.

mod cache_port;
mod checkout_port;
mod inventory_port;
mod order_port;
mod product_port;
mod user_port;

pub use cache_port::CachePort;
pub use checkout_port::CheckoutTransactionPort;
pub use inventory_port::InventoryRepository;
pub use order_port::OrderRepository;
pub use product_port::{ProductFilters, ProductPagination, ProductRepository};
pub use user_port::UserRepository;