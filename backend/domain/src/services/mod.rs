// backend/domain/src/services/mod.rs
pub mod checkout;
pub mod auth_service;
pub mod catalog_service;

pub use checkout::CheckoutService;
pub use auth_service::AuthService;
pub use catalog_service::CatalogService;
