pub struct CacheAdapter;

impl CacheAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn store_idempotency_key(&self, _key: &str, _value: &str) {
        // Placeholder for optional Redis-backed idempotency storage.
    }
}
