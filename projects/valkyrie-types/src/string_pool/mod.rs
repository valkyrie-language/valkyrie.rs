use core::fmt;
use lasso::{Key, Resolver, Spur, ThreadedRodeo};
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter, Write},
    num::NonZeroUsize,
    ops::Range,
    str::{Split, pattern::Pattern},
    sync::{Arc, LazyLock},
};

mod name_path;
mod string_id;
mod string_pool;
pub mod variable;
pub mod identifier;

pub static STRING_POOL: LazyLock<StringPool> = std::sync::LazyLock::new(|| StringPool::default());




#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NamePath {
    key: Spur,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileName {
    key: Spur,
}

impl Default for FileName {
    fn default() -> Self {
        Self { key: STRING_POOL.encode_static("") }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub struct Location {
    file: FileName,
    start: u32,
    end: u32,
}

impl FileName {
    pub fn with_range(&self, range: Range<u32>) -> Location {
        Location { file: *self, start: range.start, end: range.end }
    }
}

impl Location {
    pub fn with_range(self, range: &Range<u32>) -> Self {
        Self { file: self.file, start: range.start, end: range.end }
    }
}

pub struct StringPool {
    pool: Arc<ThreadedRodeo<Spur>>,
}
