// backend/api/src/metrics.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::RwLock;
use std::sync::LazyLock;

pub static METRICS: LazyLock<MetricsRegistry> = LazyLock::new(MetricsRegistry::new);

pub struct MetricsRegistry {
    counters: RwLock<HashMap<String, AtomicU64>>,
    labeled_counters: RwLock<HashMap<(String, String, String), AtomicU64>>,
    gauges: RwLock<HashMap<String, AtomicI64>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            labeled_counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
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

        output
    }
}
