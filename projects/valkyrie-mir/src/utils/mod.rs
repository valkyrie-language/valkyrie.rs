use crate::types::ValkyrieMetaType;
use shredder::Gc;
use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};
use xxhash_rust::xxh3::Xxh3;

pub fn primitive_type(name: &str) -> Gc<ValkyrieMetaType> {
    let mut this = ValkyrieMetaType::default();
    this.set_namepath(name);
    Gc::new(this)
}

pub fn xx_id<T: Hash>(t: &T) -> u64 {
    let mut hasher = Xxh3::default();
    t.hash(&mut hasher);
    hasher.finish()
}
