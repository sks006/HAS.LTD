// backend/infrastructure/src/tracing/mod.rs

pub mod redaction;

pub use redaction::{is_sensitive_field, RedactionLayer, SENSITIVE_FIELDS, SharedWriter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initializes the global tracing subscriber with redaction layer and format switch:
/// - RUST_LOG_FORMAT=json -> JSON formatted logs with current span and span list
/// - anything else / unset -> pretty formatted logs (dev friendly)
pub fn init_tracing(format: &str) {
    let redaction_layer = RedactionLayer::new();
    if format.eq_ignore_ascii_case("json") {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_list(true);
        let _ = tracing_subscriber::registry()
            .with(redaction_layer)
            .with(fmt_layer)
            .try_init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer().pretty();
        let _ = tracing_subscriber::registry()
            .with(redaction_layer)
            .with(fmt_layer)
            .try_init();
    }
}
