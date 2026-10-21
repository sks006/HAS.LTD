// backend/domain/src/ports/mod.rs

pub mod email_port;

pub use email_port::{EmailError, EmailMessage, EmailPort};
