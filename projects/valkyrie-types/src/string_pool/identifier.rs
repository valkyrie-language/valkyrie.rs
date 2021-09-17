use super::*;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    key: Spur,
}

impl Default for Identifier {
    fn default() -> Self {
        Self { key: STRING_POOL.encode_static("") }
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

impl From<Identifier> for Spur {
    fn from(value: Identifier) -> Self {
        value.key
    }
}
impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        STRING_POOL.decode_string(&self.key)
    }
}

impl Identifier {
    pub fn new(s: &str) -> Self {
        Self { key: STRING_POOL.encode_string(s) }
    }
    pub fn starts_with(&self, pattern: impl Pattern) -> bool {
        self.as_ref().starts_with(pattern)
    }
    pub fn contains(&self, pattern: impl Pattern) -> bool {
        self.as_ref().contains(pattern)
    }
    pub fn is_empty(&self) -> bool {
        STRING_POOL.decode_string(&self.key).is_empty()
    }
}
