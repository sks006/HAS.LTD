// backend/api/src/metrics.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::LazyLock;
use std::sync::RwLock;

pub static METRICS: LazyLock<MetricsRegistry> = LazyLock::new(MetricsRegistry::new);

pub const AUTH_ARGON2_QUEUE_DEPTH: &str = "auth_argon2_queue_depth";
pub const AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL: &str = "auth_argon2_queue_timeout_total";
pub const AUTH_MIDDLEWARE_REQUESTS_TOTAL: &str = "auth_middleware_requests_total";
pub const AUTH_MIDDLEWARE_REJECTION_TOTAL: &str = "auth_middleware_rejection_total";
pub const AUTH_MIDDLEWARE_LATENCY_SECONDS: &str = "auth_middleware_latency_seconds";
pub const AUTH_LOGIN_ATTEMPTS_TOTAL: &str = "auth_login_attempts_total";
pub const AUTH_LOGIN_FAILURES_TOTAL: &str = "auth_login_failures_total";
pub const AUTH_MFA_ATTEMPTS_TOTAL: &str = "auth_mfa_attempts_total";
pub const AUTH_MFA_FAILURES_TOTAL: &str = "auth_mfa_failures_total";
pub const AUTH_REFRESH_ROTATIONS_TOTAL: &str = "auth_refresh_rotations_total";
pub const AUTH_REFRESH_REPLAY_DETECTED_TOTAL: &str = "auth_refresh_replay_detected_total";
pub const AUTH_EMAIL_DISPATCH_FAILURE_TOTAL: &str = "auth_email_dispatch_failure_total";
pub const AUTH_EMAIL_DISPATCH_LATENCY_SECONDS: &str = "auth_email_dispatch_latency_seconds";
pub const AUTH_DB_QUERY_TOTAL: &str = "auth_db_query_total";
pub const AUTH_AUDIT_EVENTS_TOTAL: &str = "auth_audit_events_total";

pub const ALL_METRIC_NAMES: &[&str] = &[
    AUTH_ARGON2_QUEUE_DEPTH,
    AUTH_ARGON2_QUEUE_TIMEOUT_TOTAL,
    AUTH_MIDDLEWARE_REQUESTS_TOTAL,
    AUTH_MIDDLEWARE_REJECTION_TOTAL,
    AUTH_MIDDLEWARE_LATENCY_SECONDS,
    AUTH_LOGIN_ATTEMPTS_TOTAL,
    AUTH_LOGIN_FAILURES_TOTAL,
    AUTH_MFA_ATTEMPTS_TOTAL,
    AUTH_MFA_FAILURES_TOTAL,
    AUTH_REFRESH_ROTATIONS_TOTAL,
    AUTH_REFRESH_REPLAY_DETECTED_TOTAL,
    AUTH_EMAIL_DISPATCH_FAILURE_TOTAL,
    AUTH_EMAIL_DISPATCH_LATENCY_SECONDS,
    AUTH_DB_QUERY_TOTAL,
    AUTH_AUDIT_EVENTS_TOTAL,
];

pub struct Histogram {
    pub buckets: Vec<f64>,
    pub counts: Vec<AtomicU64>,
    pub sum: RwLock<f64>,
    pub count: AtomicU64,
}

impl Histogram {
    pub fn new(buckets: Vec<f64>) -> Self {
        let len = buckets.len();
        let mut counts = Vec::with_capacity(len);
        for _ in 0..len {
            counts.push(AtomicU64::new(0));
        }
        Self {
            buckets,
            counts,
            sum: RwLock::new(0.0),
            count: AtomicU64::new(0),
        }
    }

    pub fn observe(&self, value: f64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        {
            let mut sum_guard = self.sum.write().unwrap();
            *sum_guard += value;
        }
        for (i, &bound) in self.buckets.iter().enumerate() {
            if value <= bound {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, AtomicU64>>,
    labeled_counters: RwLock<HashMap<(String, String, String), AtomicU64>>,
    gauges: RwLock<HashMap<String, AtomicI64>>,
    histograms: RwLock<HashMap<String, Histogram>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let mut histograms = HashMap::new();
        // Phase 7 §1.i defined buckets
        histograms.insert(
            AUTH_MIDDLEWARE_LATENCY_SECONDS.to_string(),
            Histogram::new(vec![0.0001, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.05]),
        );
        histograms.insert(
            AUTH_EMAIL_DISPATCH_LATENCY_SECONDS.to_string(),
            Histogram::new(vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0]),
        );

        Self {
            counters: RwLock::new(HashMap::new()),
            labeled_counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(histograms),
        }
    }

    pub fn counter_inc(&self, name: &str) {
        let read = self.counters.read().unwrap();
        if let Some(counter) = read.get(name) {
            counter.fetch_add(1, Ordering::Relaxed);
            return;
        }
        drop(read);
        let mut write = self.counters.write().unwrap();
        let counter = write.entry(name.to_string()).or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn counter_inc_labeled(&self, name: &str, label_key: &str, label_val: &str) {
        let key = (name.to_string(), label_key.to_string(), label_val.to_string());
        let read = self.labeled_counters.read().unwrap();
        if let Some(counter) = read.get(&key) {
            counter.fetch_add(1, Ordering::Relaxed);
            return;
        }
        drop(read);
        let mut write = self.labeled_counters.write().unwrap();
        let counter = write.entry(key).or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn gauge_inc(&self, name: &str) {
        let read = self.gauges.read().unwrap();
        if let Some(gauge) = read.get(name) {
            gauge.fetch_add(1, Ordering::Relaxed);
            return;
        }
        drop(read);
        let mut write = self.gauges.write().unwrap();
        let gauge = write.entry(name.to_string()).or_insert_with(|| AtomicI64::new(0));
        gauge.fetch_add(1, Ordering::Relaxed);
    }

    pub fn gauge_dec(&self, name: &str) {
        let read = self.gauges.read().unwrap();
        if let Some(gauge) = read.get(name) {
            gauge.fetch_sub(1, Ordering::Relaxed);
            return;
        }
        drop(read);
        let mut write = self.gauges.write().unwrap();
        let gauge = write.entry(name.to_string()).or_insert_with(|| AtomicI64::new(0));
        gauge.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn histogram_observe(&self, name: &str, value: f64) {
        let read = self.histograms.read().unwrap();
        if let Some(hist) = read.get(name) {
            hist.observe(value);
            return;
        }
        drop(read);
        let mut write = self.histograms.write().unwrap();
        let hist = write.entry(name.to_string()).or_insert_with(|| {
            let buckets = match name {
                AUTH_MIDDLEWARE_LATENCY_SECONDS => vec![0.0001, 0.0005, 0.001, 0.002, 0.005, 0.01, 0.05],
                AUTH_EMAIL_DISPATCH_LATENCY_SECONDS => vec![0.05, 0.1, 0.25, 0.5, 1.0, 2.0, 5.0],
                _ => vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
            };
            Histogram::new(buckets)
        });
        hist.observe(value);
    }

    pub fn render_prometheus(&self) -> String {
        let mut output = String::new();

        // Render simple counters
        {
            let counters = self.counters.read().unwrap();
            for (name, val) in counters.iter() {
                output.push_str(&format!("# TYPE {} counter\n", name));
                output.push_str(&format!("{} {}\n", name, val.load(Ordering::Relaxed)));
            }
        }

        // Render labeled counters
        {
            let labeled = self.labeled_counters.read().unwrap();
            let mut by_metric: HashMap<String, Vec<((String, String), u64)>> = HashMap::new();
            for ((name, l_key, l_val), val) in labeled.iter() {
                by_metric.entry(name.clone()).or_default().push(((l_key.clone(), l_val.clone()), val.load(Ordering::Relaxed)));
            }

            for (name, series) in by_metric.iter() {
                output.push_str(&format!("# TYPE {} counter\n", name));
                for ((l_key, l_val), val) in series {
                    output.push_str(&format!("{name}{{{}=\"{}\"}} {}\n", l_key, l_val, val));
                }
            }
        }

        // Render gauges
        {
            let gauges = self.gauges.read().unwrap();
            for (name, val) in gauges.iter() {
                output.push_str(&format!("# TYPE {} gauge\n", name));
                output.push_str(&format!("{} {}\n", name, val.load(Ordering::Relaxed)));
            }
        }

        // Render histograms
        {
            let histograms = self.histograms.read().unwrap();
            for (name, hist) in histograms.iter() {
                output.push_str(&format!("# TYPE {} histogram\n", name));
                let total_count = hist.count.load(Ordering::Relaxed);
                for (i, &bound) in hist.buckets.iter().enumerate() {
                    let count = hist.counts[i].load(Ordering::Relaxed);
                    output.push_str(&format!("{}_bucket{{le=\"{}\"}} {}\n", name, bound, count));
                }
                output.push_str(&format!("{}_bucket{{le=\"+Inf\"}} {}\n", name, total_count));
                let sum = *hist.sum.read().unwrap();
                output.push_str(&format!("{}_sum {}\n", name, sum));
                output.push_str(&format!("{}_count {}\n", name, total_count));
            }
        }

        output
    }
}
