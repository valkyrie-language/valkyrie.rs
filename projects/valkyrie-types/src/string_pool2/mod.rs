use crate::bindings::exports::valkyrie::valkyrie_legacy::string_pool::{Guest, GuestStringPool};
use lasso::{Spur, ThreadedRodeo};
use std::sync::{Arc, LazyLock};

pub mod identifier;

pub struct StringContext {}

impl Guest for StringContext {
    type StringPool = StringPool;
    type Identifier = identifier::Identifier;
}

pub static STRING_POOL: LazyLock<StringPool> = std::sync::LazyLock::new(|| StringPool::default());

pub struct StringPool {
    pool: Arc<ThreadedRodeo<Spur>>,
}

impl GuestStringPool for StringPool {}

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
