use core::fmt;
use lasso::{Key, Resolver, Spur, ThreadedRodeo};
use std::{
    borrow::Cow,
    fmt::{Debug, Write},
    num::NonZeroUsize,
    str::Split,
    sync::{Arc, LazyLock},
};

mod name_path;
mod string_id;
mod string_pool;
pub mod variable;

pub static STRING_POOL: LazyLock<StringPool> = std::sync::LazyLock::new(|| StringPool::default());

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    key: Spur,
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

pub struct StringPool {
    pool: Arc<ThreadedRodeo<Spur>>,
}
