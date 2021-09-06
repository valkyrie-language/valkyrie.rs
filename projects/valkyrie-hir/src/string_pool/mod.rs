use core::fmt;
use lasso::{Key, Spur, Resolver, ThreadedRodeo};
use std::{
    borrow::Cow,
    fmt::{Debug, Write},
    num::NonZeroUsize,
    sync::{Arc, LazyLock},
};
use std::str::Split;

mod string_id;
mod string_pool;
mod name_path;

pub static STRING_POOL: LazyLock<StringPool> = std::sync::LazyLock::new(|| StringPool::default());

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    key: Spur,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct NamePath {
    key: Spur,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileName {
    key: Spur,
}

pub struct Location {
    file: FileName,
    start: u32,
    end: u32,
}

pub struct StringPool {
    pool: Arc<ThreadedRodeo<Spur>>,
}
