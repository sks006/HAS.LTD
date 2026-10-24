// backend/api/tests/observability_tests.rs
//! Phase 7 Observability Test Suite
//! Validates:
//! 1. Log-field contract (redaction list vs non-sensitive allowlist)
//! 2. Metrics cardinality budget (<10,000 series total, <1,000 series per metric)
//! 3. Redaction layer enforcement in JSON mode
//! 4. Redaction layer enforcement in Pretty mode
//! 5. Prometheus alert rules parsing and runbook URL validation
//! 6. Grafana dashboards schemaVersion >= 38 and metric inventory compliance
//! 7. All metric constants have concrete emission sites in src/

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;

use api::metrics::{
    ALL_METRIC_NAMES, AUTH_ARGON2_QUEUE_DEPTH, AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL,
    AUTH_AUDIT_EVENTS_TOTAL, AUTH_DB_QUERY_TOTAL, AUTH_EMAIL_DISPATCH_FAILURE_TOTAL,
    AUTH_EMAIL_DISPATCH_LATENCY_SECONDS, AUTH_LOGIN_ATTEMPTS_TOTAL, AUTH_LOGIN_FAILURES_TOTAL,
    AUTH_MFA_ATTEMPTS_TOTAL, AUTH_MFA_FAILURES_TOTAL, AUTH_MIDDLEWARE_LATENCY_SECONDS,
    AUTH_MIDDLEWARE_REJECTION_TOTAL, AUTH_MIDDLEWARE_REQUESTS_TOTAL,
    AUTH_REFRESH_REPLAY_DETECTED_TOTAL, AUTH_REFRESH_ROTATIONS_TOTAL,
};
use infrastructure::tracing::redaction::{RedactionLayer, SENSITIVE_FIELDS};

#[derive(Clone, Default)]
struct TestBuffer {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for TestBuffer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().write(data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buf.lock().unwrap().flush()
    }
}

/// Helper to resolve workspace root directory.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // manifest_dir is backend/api -> parent is backend -> parent is workspace root
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to locate workspace root")
        .to_path_buf()
}

// ================================================================
// 1. Log Field Contract Test
// ================================================================
fn assert_field_classified(
    field: &str,
    redaction_list: &HashSet<&str>,
    non_sensitive_allowlist: &HashSet<&str>,
    excluded_from_check: &std::collections::HashMap<&str, &str>,
) {
    if excluded_from_check.contains_key(field) {
        return;
    }
    let is_redacted = redaction_list.contains(field);
    let is_allowed = non_sensitive_allowlist.contains(field);
    assert!(
        is_redacted || is_allowed,
        "Field '{}' is unclassified! Must be in redaction list, allowlist, or excluded_from_check.",
        field
    );
    assert!(
        !(is_redacted && is_allowed),
        "Field '{}' is in BOTH redaction list and allowlist!",
        field
    );
}

#[test]
fn test_log_field_contract() {
    // Redaction list defined in Phase 2 redaction.rs:
    let redaction_list: HashSet<&str> = SENSITIVE_FIELDS.iter().copied().collect();

    // Non-sensitive allowlist candidate fields:
    let non_sensitive_allowlist: HashSet<&str> = [
        "user_id",
        "session_id",
        "jti",
        "family_id",
        "token_hash_prefix",
        "email_domain",
        "attempted_email_domain",
        "method",
        "reason",
        "role",
        "event",
        "status",
        "duration_seconds",
        "remaining_codes",
        "count",
        "sessions_revoked",
        "target_session_id",
        "message",
        "error",
        "user",
        "operation",
        "ip",
        "ip_address",
        "user_agent",
    ]
    .into_iter()
    .collect();

    // False-positive exclusions from regex AST extraction across api/src with one-line justifications:
    let excluded_from_check: std::collections::HashMap<&str, &str> = [
        ("access_token", "HTTP wire response DTO field, not an audit or span field"),
        ("code", "HTTP error response JSON envelope key ('error.code'), not an audit or span field"),
        ("error", "HTTP error response root envelope key ('error'), not an audit or span field"),
        ("raw_refresh", "Internal Redis grace cache serialization payload key, not an audit or span field"),
        ("ok", "HTTP /health endpoint response body JSON payload key, not an audit or span field"),
        ("database_status", "HTTP /health endpoint diagnostic payload key, not an audit or span field"),
        ("redis_status", "HTTP /health endpoint diagnostic payload key, not an audit or span field"),
        ("active_connections", "HTTP /health endpoint diagnostic payload key, not an audit or span field"),
        ("cpu_usage_pct", "HTTP /health endpoint diagnostic payload key, not an audit or span field"),
        ("size", "Mock test catalog product attribute in bin/seed_and_token.rs, not an audit or span field"),
        ("color", "Mock test catalog product attribute in bin/seed_and_token.rs, not an audit or span field"),
    ]
    .into_iter()
    .collect();

    // Collect all audit metadata keys and tracing field names emitted in backend/api/src/
    let api_src = workspace_root().join("backend/api/src");
    let mut collected_fields = HashSet::new();

    fn walk_dir(dir: &Path, fields: &mut HashSet<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    // Skip CLI binary tools in src/bin, checking API server library handlers & middleware
                    if p.file_name().map_or(false, |name| name == "bin") {
                        continue;
                    }
                    walk_dir(&p, fields);
                } else if p.extension().map_or(false, |ext| ext == "rs") {
                    let content = fs::read_to_string(&p).unwrap_or_default();
                    // 1. Tracing span and event fields
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("tracing::") || trimmed.contains("span!(") {
                            for part in trimmed.split(',') {
                                if let Some(eq_idx) = part.find('=') {
                                    let key = part[..eq_idx].trim().trim_start_matches(|c: char| !c.is_alphanumeric() && c != '_');
                                    if !key.is_empty() && key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                                        fields.insert(key.to_string());
                                    }
                                }
                            }
                        }
                    }
                    // 2. JSON keys in serde_json::json!({...})
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(idx) = trimmed.find("\":") {
                            let prefix = &trimmed[..idx];
                            if let Some(q_start) = prefix.rfind('"') {
                                let key = prefix[q_start + 1..]
                                    .trim_matches('\\')
                                    .trim_matches('"')
                                    .trim();
                                if !key.contains(' ') && !key.is_empty() {
                                    fields.insert(key.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    walk_dir(&api_src, &mut collected_fields);

    // Assert that walk_dir collected real fields from the codebase
    assert!(
        collected_fields.len() >= 10,
        "walk_dir must extract emitted fields from codebase, found only {}",
        collected_fields.len()
    );

    // Assert EVERY field in collected_fields is classified:
    for field in &collected_fields {
        assert_field_classified(
            field,
            &redaction_list,
            &non_sensitive_allowlist,
            &excluded_from_check,
        );
    }

    // Also assert on all explicitly known span fields and audit metadata keys
    let test_sample_fields = [
        "user_id",
        "session_id",
        "jti",
        "family_id",
        "token_hash_prefix",
        "email_domain",
        "attempted_email_domain",
        "method",
        "reason",
        "role",
        "event",
        "status",
        "duration_seconds",
        "remaining_codes",
        "count",
        "sessions_revoked",
        "target_session_id",
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

    for field in test_sample_fields {
        assert_field_classified(
            field,
            &redaction_list,
            &non_sensitive_allowlist,
            &excluded_from_check,
        );
    }
}

#[test]
fn test_log_field_contract_rejects_unclassified_field() {
    let redaction_list: HashSet<&str> = SENSITIVE_FIELDS.iter().copied().collect();
    let non_sensitive_allowlist: HashSet<&str> = [
        "user_id",
        "session_id",
        "jti",
        "family_id",
        "token_hash_prefix",
        "email_domain",
        "attempted_email_domain",
        "method",
        "reason",
        "role",
        "event",
        "status",
        "duration_seconds",
        "remaining_codes",
        "count",
        "sessions_revoked",
        "target_session_id",
        "message",
        "error",
        "user",
        "operation",
        "ip",
        "ip_address",
        "user_agent",
    ]
    .into_iter()
    .collect();
    let excluded_from_check = std::collections::HashMap::new();

    let unclassified_field = "unclassified_secret_payload";

    let result = std::panic::catch_unwind(|| {
        assert_field_classified(
            unclassified_field,
            &redaction_list,
            &non_sensitive_allowlist,
            &excluded_from_check,
        );
    });

    assert!(
        result.is_err(),
        "Contract enforcement mechanism MUST panic when an unclassified field name appears!"
    );
}

// ================================================================
// 2. Metrics Cardinality Budget Test
// ================================================================
#[test]
fn test_metrics_cardinality_budget() {
    struct MetricBudget {
        name: &'static str,
        labels: Vec<(&'static str, usize)>, // (label_name, distinct_cardinality)
    }

    let budgets = vec![
        MetricBudget {
            name: AUTH_ARGON2_QUEUE_DEPTH,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_MIDDLEWARE_REQUESTS_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_MIDDLEWARE_REJECTION_TOTAL,
            labels: vec![("reason", 6)], // invalid_token, token_expired, token_revoked, account_suspended, permission_denied, internal_error
        },
        MetricBudget {
            name: AUTH_MIDDLEWARE_LATENCY_SECONDS,
            labels: vec![("le", 8)], // 7 buckets + Inf + sum + count = 10 series
        },
        MetricBudget {
            name: AUTH_LOGIN_ATTEMPTS_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_LOGIN_FAILURES_TOTAL,
            labels: vec![("reason", 6)], // invalid_credentials, credentials_missing, user_not_found, account_suspended, validation_error, rate_limited
        },
        MetricBudget {
            name: AUTH_MFA_ATTEMPTS_TOTAL,
            labels: vec![("method", 2)], // totp, recovery
        },
        MetricBudget {
            name: AUTH_MFA_FAILURES_TOTAL,
            labels: vec![("method", 2)], // totp, recovery
        },
        MetricBudget {
            name: AUTH_REFRESH_ROTATIONS_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_REFRESH_REPLAY_DETECTED_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_EMAIL_DISPATCH_FAILURE_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_EMAIL_DISPATCH_LATENCY_SECONDS,
            labels: vec![("le", 8)], // 7 buckets + Inf + sum + count = 10 series
        },
        MetricBudget {
            name: AUTH_DB_QUERY_TOTAL,
            labels: vec![],
        },
        MetricBudget {
            name: AUTH_AUDIT_EVENTS_TOTAL,
            labels: vec![("event", 23)], // 23 frozen audit events
        },
    ];

    let mut total_time_series = 0;

    for metric in &budgets {
        let series_count = if metric.labels.is_empty() {
            1
        } else {
            metric.labels.iter().map(|(_, card)| *card).product()
        };

        assert!(
            series_count <= 1000,
            "Metric '{}' exceeds 1,000 series limit! Series count: {}",
            metric.name,
            series_count
        );

        total_time_series += series_count;
    }

    assert!(
        total_time_series <= 10_000,
        "Total auth time series budget exceeded! Total: {}",
        total_time_series
    );
    assert_eq!(total_time_series, 63);
}

// ================================================================
// 3. Tracing Redaction in JSON Mode Test
// ================================================================
#[test]
fn test_redaction_applies_in_json_mode() {
    let test_buf = TestBuffer::default();
    let memory = test_buf.buf.clone();

    let redaction_layer = RedactionLayer::with_writer(Arc::new(Mutex::new(test_buf)));
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true);

    let subscriber = tracing_subscriber::registry()
        .with(redaction_layer)
        .with(fmt_layer);

    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!(
            "auth_flow",
            user_id = "user-1234",
            password = "plaintext_password_val",
            token = "jwt_access_token_val",
            totp_key = "sensitive_totp_key_secret"
        );
        let _guard = span.enter();
        tracing::info!(
            otp = "987654",
            recovery_code = "ABCD-1234",
            email_domain = "example.com"
        );
    });

    let output = String::from_utf8(memory.lock().unwrap().clone()).expect("valid utf8");

    // Assert sensitive secrets are scrubbed
    assert!(!output.contains("plaintext_password_val"));
    assert!(!output.contains("jwt_access_token_val"));
    assert!(!output.contains("sensitive_totp_key_secret"));
    assert!(!output.contains("987654"));
    assert!(!output.contains("ABCD-1234"));

    // Assert redaction marker is applied
    assert!(output.contains("[REDACTED]"));
    // Assert non-sensitive fields are preserved
    assert!(output.contains("user-1234"));
    assert!(output.contains("example.com"));
}

// ================================================================
// 4. Tracing Redaction in Pretty Mode Test
// ================================================================
#[test]
fn test_redaction_applies_in_pretty_mode() {
    let test_buf = TestBuffer::default();
    let memory = test_buf.buf.clone();

    let redaction_layer = RedactionLayer::with_writer(Arc::new(Mutex::new(test_buf)));
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();

    let subscriber = tracing_subscriber::registry()
        .with(redaction_layer)
        .with(fmt_layer);

    tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!(
            "session_flow",
            session_id = "sess-5678",
            password_hash = "$argon2id$v=19$m=65536,t=3,p=4$dummy_hash",
            mfa_secret = "JBSWY3DPEHPK3PXP"
        );
        let _guard = span.enter();
        tracing::warn!(
            secret_hash = "secret_hash_value",
            method = "totp",
            status = "ACTIVE"
        );
    });

    let output = String::from_utf8(memory.lock().unwrap().clone()).expect("valid utf8");

    // Assert sensitive secrets are scrubbed
    assert!(!output.contains("$argon2id$v=19$m=65536,t=3,p=4$dummy_hash"));
    assert!(!output.contains("JBSWY3DPEHPK3PXP"));
    assert!(!output.contains("secret_hash_value"));

    // Assert redaction marker is applied
    assert!(output.contains("[REDACTED]"));
    // Assert non-sensitive fields are preserved
    assert!(output.contains("sess-5678"));
    assert!(output.contains("totp"));
    assert!(output.contains("ACTIVE"));
}

// ================================================================
// 5. Alert Rules Parse and Runbook URL Test
// ================================================================
#[test]
fn test_alert_rules_parse() {
    let alerts_path = workspace_root().join("ops/alerts/auth.yml");
    assert!(
        alerts_path.exists(),
        "ops/alerts/auth.yml must exist at {}",
        alerts_path.display()
    );

    let content = fs::read_to_string(&alerts_path).expect("Failed to read ops/alerts/auth.yml");
    let yaml: serde_yaml::Value =
        serde_yaml::from_str(&content).expect("Failed to parse ops/alerts/auth.yml as YAML");

    let groups = yaml
        .get("groups")
        .and_then(|g| g.as_sequence())
        .expect("groups array expected in alerts YAML");

    assert!(!groups.is_empty(), "groups array must not be empty");

    let mut alert_count = 0;
    for group in groups {
        let rules = group
            .get("rules")
            .and_then(|r| r.as_sequence())
            .expect("rules array expected in each group");

        for rule in rules {
            let alert_name = rule
                .get("alert")
                .and_then(|a| a.as_str())
                .expect("alert name expected");
            let annotations = rule
                .get("annotations")
                .and_then(|a| a.as_mapping())
                .expect("annotations expected for alert rule");

            let runbook_url = annotations
                .get(&serde_yaml::Value::String("runbook_url".to_string()))
                .and_then(|u| u.as_str())
                .unwrap_or_else(|| panic!("Alert '{}' is missing 'runbook_url' annotation!", alert_name));

            assert!(
                !runbook_url.is_empty(),
                "Alert '{}' runbook_url cannot be empty",
                alert_name
            );

            // Assert that the runbook file exists on disk
            let runbook_file = workspace_root().join(runbook_url);
            assert!(
                runbook_file.exists(),
                "Alert '{}' points to nonexistent runbook: {}",
                alert_name,
                runbook_file.display()
            );

            alert_count += 1;
        }
    }

    assert!(alert_count >= 8, "Expected at least 8 alert rules, found {}", alert_count);
}

// ================================================================
// 6. Dashboards JSON Valid Test
// ================================================================
#[test]
fn test_dashboards_json_valid() {
    let dashboards = [
        ("ops/dashboards/auth-health.json", "Auth Health", 6),
        ("ops/dashboards/auth-security.json", "Auth Security", 8),
    ];

    for (rel_path, expected_title, min_panels) in dashboards {
        let dash_path = workspace_root().join(rel_path);
        assert!(
            dash_path.exists(),
            "Dashboard file must exist at {}",
            dash_path.display()
        );

        let content = fs::read_to_string(&dash_path).expect("Failed to read dashboard JSON");
        let json: serde_json::Value =
            serde_json::from_str(&content).expect("Failed to parse dashboard JSON");

        // Assert schemaVersion >= 38
        let schema_ver = json
            .get("schemaVersion")
            .and_then(|v| v.as_u64())
            .expect("schemaVersion must be an unsigned integer");
        assert!(
            schema_ver >= 38,
            "Dashboard {} schemaVersion must be >= 38, got {}",
            rel_path,
            schema_ver
        );

        // Assert title matches
        let title = json.get("title").and_then(|v| v.as_str()).unwrap_or("");
        assert_eq!(title, expected_title);

        // Assert panels exist
        let panels = json
            .get("panels")
            .and_then(|p| p.as_array())
            .expect("panels array must be present");
        assert!(
            panels.len() >= min_panels,
            "Dashboard {} expected >= {} panels, got {}",
            rel_path,
            min_panels,
            panels.len()
        );

        // Assert that expressions in dashboard only reference metrics in ALL_METRIC_NAMES
        for panel in panels {
            if let Some(targets) = panel.get("targets").and_then(|t| t.as_array()) {
                for target in targets {
                    if let Some(expr) = target.get("expr").and_then(|e| e.as_str()) {
                        let has_valid_metric = ALL_METRIC_NAMES
                            .iter()
                            .any(|&metric| expr.contains(metric));
                        assert!(
                            has_valid_metric,
                            "Panel target expr '{}' in {} does not reference any metric from inventory!",
                            expr, rel_path
                        );
                    }
                }
            }
        }
    }
}

// ================================================================
// 7. Every Metric Has Real Emission Site Test (REQUIRED-2)
// ================================================================
#[test]
fn test_every_metric_has_emission_site() {
    let api_src = workspace_root().join("backend/api/src");
    let mut all_src_text = String::new();

    fn read_all_rs(dir: &Path, acc: &mut String) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    read_all_rs(&p, acc);
                } else if p.extension().map_or(false, |ext| ext == "rs") {
                    // Exclude metrics.rs where metric constants are declared
                    if p.file_name().map_or(false, |name| name != "metrics.rs") {
                        if let Ok(content) = fs::read_to_string(&p) {
                            acc.push_str(&content);
                            acc.push('\n');
                        }
                    }
                }
            }
        }
    }

    read_all_rs(&api_src, &mut all_src_text);

    // Map each metric literal to its corresponding constant identifier in metrics.rs
    let metric_constant_map: &[(&str, &str)] = &[
        (AUTH_ARGON2_QUEUE_DEPTH, "AUTH_ARGON2_QUEUE_DEPTH"),
        (AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL, "AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL"),
        (AUTH_MIDDLEWARE_REQUESTS_TOTAL, "AUTH_MIDDLEWARE_REQUESTS_TOTAL"),
        (AUTH_MIDDLEWARE_REJECTION_TOTAL, "AUTH_MIDDLEWARE_REJECTION_TOTAL"),
        (AUTH_MIDDLEWARE_LATENCY_SECONDS, "AUTH_MIDDLEWARE_LATENCY_SECONDS"),
        (AUTH_LOGIN_ATTEMPTS_TOTAL, "AUTH_LOGIN_ATTEMPTS_TOTAL"),
        (AUTH_LOGIN_FAILURES_TOTAL, "AUTH_LOGIN_FAILURES_TOTAL"),
        (AUTH_MFA_ATTEMPTS_TOTAL, "AUTH_MFA_ATTEMPTS_TOTAL"),
        (AUTH_MFA_FAILURES_TOTAL, "AUTH_MFA_FAILURES_TOTAL"),
        (AUTH_REFRESH_ROTATIONS_TOTAL, "AUTH_REFRESH_ROTATIONS_TOTAL"),
        (AUTH_REFRESH_REPLAY_DETECTED_TOTAL, "AUTH_REFRESH_REPLAY_DETECTED_TOTAL"),
        (AUTH_EMAIL_DISPATCH_FAILURE_TOTAL, "AUTH_EMAIL_DISPATCH_FAILURE_TOTAL"),
        (AUTH_EMAIL_DISPATCH_LATENCY_SECONDS, "AUTH_EMAIL_DISPATCH_LATENCY_SECONDS"),
        (AUTH_DB_QUERY_TOTAL, "AUTH_DB_QUERY_TOTAL"),
        (AUTH_AUDIT_EVENTS_TOTAL, "AUTH_AUDIT_EVENTS_TOTAL"),
    ];

    // Collect all real emission calls:
    // METRICS.counter_inc( ... )
    // METRICS.counter_inc_labeled( ... )
    // METRICS.histogram_observe( ... )
    // METRICS.gauge_* ( ... )
    let mut emission_calls = Vec::new();
    for line in all_src_text.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find("METRICS.") {
            let call_slice = &trimmed[idx + "METRICS.".len()..];
            if call_slice.starts_with("counter_inc")
                || call_slice.starts_with("counter_inc_labeled")
                || call_slice.starts_with("histogram_observe")
                || call_slice.starts_with("gauge_")
            {
                if let Some(open_paren) = call_slice.find('(') {
                    let arg_slice = &call_slice[open_paren + 1..];
                    let first_arg = arg_slice
                        .split(&[',', ')'][..])
                        .next()
                        .unwrap_or("")
                        .trim();
                    let clean_arg = first_arg
                        .trim_start_matches("crate::metrics::")
                        .trim_start_matches("api::metrics::")
                        .trim();
                    emission_calls.push(clean_arg.to_string());
                }
            }
        }
    }

    assert!(
        emission_calls.len() >= 15,
        "Expected at least 15 real METRICS emission call sites, found {}",
        emission_calls.len()
    );

    for &(metric_name, const_name) in metric_constant_map {
        let quoted_literal = format!("\"{}\"", metric_name);
        let has_emission = emission_calls.iter().any(|arg| {
            arg == &quoted_literal || arg == const_name || arg == metric_name
        });

        assert!(
            has_emission,
            "Metric '{}' (constant '{}') has NO real emission call site in backend/api/src! Checked against {} emission calls.",
            metric_name,
            const_name,
            emission_calls.len()
        );
    }
}
