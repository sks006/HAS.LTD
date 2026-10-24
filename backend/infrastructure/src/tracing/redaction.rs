// backend/infrastructure/src/tracing/redaction.rs

use std::fmt;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

pub const REDACTED_MARKER: &str = "[REDACTED]";

pub const SENSITIVE_FIELDS: &[&str] = &[
    "password",
    "password_hash",
    "secret",
    "secret_hash",
    "token",
    "token_hash",
    "otp",
    "otp_hash",
    "mfa_secret",
    "totp_secret",
    "recovery_code",
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "email_pepper",
    "totp_key",
];

/// Checks whether a tracing field name matches any known sensitive credentials identifier.
pub fn is_sensitive_field(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    SENSITIVE_FIELDS.iter().any(|&f| f == lower)
}

/// In-memory storage for redacted span attributes attached via tracing extensions.
#[derive(Debug, Clone)]
pub struct RedactedFields {
    pub fields: Vec<(String, String)>,
}

/// Visitor that intercepts fields and substitutes sensitive values with `[REDACTED]`.
struct RedactingVisitor {
    entries: Vec<(String, String)>,
}

impl RedactingVisitor {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Visit for RedactingVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if is_sensitive_field(field.name()) {
            self.entries
                .push((field.name().to_string(), REDACTED_MARKER.to_string()));
        } else {
            self.entries
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if is_sensitive_field(field.name()) {
            self.entries
                .push((field.name().to_string(), REDACTED_MARKER.to_string()));
        } else {
            self.entries
                .push((field.name().to_string(), value.to_string()));
        }
    }
}

pub type SharedWriter = Arc<Mutex<dyn Write + Send + Sync + 'static>>;

/// Tracing subscriber layer that intercepts spans and events, scrubs all sensitive credentials,
/// and writes sanitized logs to a configurable writer (standard output or test buffer).
pub struct RedactionLayer {
    writer: SharedWriter,
}

impl RedactionLayer {
    /// Creates a new RedactionLayer outputting to standard output.
    pub fn new() -> Self {
        Self {
            writer: Arc::new(Mutex::new(std::io::stdout())),
        }
    }

    /// Creates a new RedactionLayer with a custom shared writer (useful for capturing in unit tests).
    pub fn with_writer(writer: SharedWriter) -> Self {
        Self { writer }
    }
}

impl Default for RedactionLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for RedactionLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut visitor = RedactingVisitor::new();
        attrs.record(&mut visitor);

        if let Some(span_ref) = ctx.span(id) {
            span_ref.extensions_mut().insert(RedactedFields {
                fields: visitor.entries.clone(),
            });

            // Write formatted span creation message with redacted fields
            if let Ok(mut writer) = self.writer.lock() {
                let mut field_str = String::new();
                for (k, v) in &visitor.entries {
                    if !field_str.is_empty() {
                        field_str.push(' ');
                    }
                    field_str.push_str(&format!("{}={}", k, v));
                }
                let _ = writeln!(
                    writer,
                    "SPAN [{}]: {} {{ {} }}",
                    attrs.metadata().level(),
                    attrs.metadata().name(),
                    field_str
                );
            }
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let mut visitor = RedactingVisitor::new();
        values.record(&mut visitor);

        if let Some(span_ref) = ctx.span(id) {
            let mut extensions = span_ref.extensions_mut();
            if let Some(existing) = extensions.get_mut::<RedactedFields>() {
                existing.fields.extend(visitor.entries);
            } else {
                extensions.insert(RedactedFields {
                    fields: visitor.entries,
                });
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut visitor = RedactingVisitor::new();
        event.record(&mut visitor);

        if let Ok(mut writer) = self.writer.lock() {
            let mut span_context = String::new();
            if let Some(span_ref) = ctx.lookup_current() {
                if let Some(redacted) = span_ref.extensions().get::<RedactedFields>() {
                    for (k, v) in &redacted.fields {
                        if !span_context.is_empty() {
                            span_context.push(' ');
                        }
                        span_context.push_str(&format!("{}={}", k, v));
                    }
                }
            }

            let mut event_fields = String::new();
            for (k, v) in &visitor.entries {
                if !event_fields.is_empty() {
                    event_fields.push(' ');
                }
                event_fields.push_str(&format!("{}={}", k, v));
            }

            let _ = writeln!(
                writer,
                "EVENT [{}]: span={{{}}} fields={{{}}}",
                event.metadata().level(),
                span_context,
                event_fields
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::layer::SubscriberExt;

    #[derive(Clone, Default)]
    struct MemoryWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for MemoryWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.buffer.lock().unwrap().flush()
        }
    }

    #[test]
    fn test_redaction_scrubs_password_in_span() {
        let memory = MemoryWriter::default();
        let buffer_clone = memory.buffer.clone();

        let layer = RedactionLayer::with_writer(Arc::new(Mutex::new(memory)));
        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(
                "user_login",
                user = "alice",
                password = "hunter2",
                token = "secret_jwt_value"
            );
            let _enter = span.enter();
            tracing::info!(otp = "123456", message = "authenticating");
        });

        let output = String::from_utf8(buffer_clone.lock().unwrap().clone()).expect("valid utf8");

        // Assert sensitive secrets are NEVER present in output
        assert!(!output.contains("hunter2"), "Plaintext password must be redacted!");
        assert!(!output.contains("secret_jwt_value"), "Plaintext token must be redacted!");
        assert!(!output.contains("123456"), "Plaintext OTP must be redacted!");

        // Assert redaction marker is applied
        assert!(output.contains("[REDACTED]"), "Redacted marker must be present");
        assert!(output.contains("user=\"alice\"") || output.contains("user=alice"), "Non-sensitive fields must remain");
    }
}
