use super::*;

impl Default for StringPool {
    fn default() -> Self {
        Self { pool: Arc::new(ThreadedRodeo::new()) }
    }
}

impl StringPool {
    pub fn encode_static(&self, s: &'static str) -> Spur {
        self.pool.get_or_intern_static(s)
    }
    pub fn encode_string(&self, s: &str) -> Spur {
        self.pool.get_or_intern(s)
    }
    pub fn decode_string(&self, string_id: &Spur) -> &str {
        self.pool.try_resolve(string_id).unwrap_or_else(|| {
            tracing::error!("StringPool: failed to resolve string");
            ""
        })
    }
}
