use super::*;
use std::fmt::Formatter;

impl Debug for NamePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl From<Spur> for NamePath {
    fn from(value: Spur) -> Self {
        Self { key: value }
    }
}

impl NamePath {
    pub fn new(path: &[&str]) -> Self {
        // ⸬
        let s = STRING_POOL.encode_string(&path.join("∷"));
        Self { key: s }
    }
    pub fn segments(&self) -> Split<char> {
        STRING_POOL.decode_string(&self.key).split('∷')
    }
    pub fn as_str(&self) -> &str {
        STRING_POOL.decode_string(&self.key)
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}
