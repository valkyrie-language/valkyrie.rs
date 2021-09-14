use core::fmt;
use lasso::{Key, Resolver, Spur, ThreadedRodeo};
use std::{
    borrow::Cow,
    fmt::{Debug, Write},
    num::NonZeroUsize,
    ops::Range,
    str::Split,
    sync::{Arc, LazyLock},
};
use std::fmt::{Display, Formatter};
use std::str::pattern::Pattern;

mod name_path;
mod string_id;
mod string_pool;
pub mod variable;

pub static STRING_POOL: LazyLock<StringPool> = std::sync::LazyLock::new(|| StringPool::default());

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    key: Spur,
}
impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        STRING_POOL.decode_string(&self.key)
    }
}

impl From<Identifier> for Spur {
    fn from(value: Identifier) -> Self {
        value.key
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl Identifier {
    pub fn new(s: &str) -> Self {
        Self {
            key: STRING_POOL.encode_string(s),
        }
    }
    pub fn starts_with(&self, pattern: impl Pattern) -> bool {
        self.as_ref().starts_with(pattern)
    }
}

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
