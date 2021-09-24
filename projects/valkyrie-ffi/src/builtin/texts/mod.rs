use crate::{packages::ids::VALKYRIE_UNIVERSE, utils::xx_id};
use dashmap::mapref::one::Ref;
use std::fmt::{Debug, Display, Formatter};

mod display;

const MISSING_TEXT: &'static str = "<MISS_TEXT>";

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringID {
    hash: u64,
}

pub struct ValkyrieString<'i> {
    pub(crate) ptr: Option<Ref<'i, StringID, Vec<u8>>>,
}

impl StringID {
    pub fn new<S>(s: S) -> Self
    where
        S: Into<String>,
    {
        let text = s.into();
        let hash = xx_id(&text);
        VALKYRIE_UNIVERSE.texts.insert(Self { hash }, text.into_bytes());
        Self { hash }
    }
    pub fn as_string(&self) -> ValkyrieString {
        ValkyrieString { ptr: VALKYRIE_UNIVERSE.texts.get(self) }
    }
}
