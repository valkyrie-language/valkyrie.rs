use super::*;

impl Debug for StringID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.as_string(), f)
    }
}
impl Display for StringID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.as_string(), f)
    }
}
impl<'i> Debug for ValkyrieString<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.ptr {
            Some(s) => f.write_str(&String::from_utf8_lossy(s.value())),
            None => f.write_str(MISSING_TEXT),
        }
    }
}
impl<'i> Display for ValkyrieString<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.ptr {
            Some(s) => f.write_str(&String::from_utf8_lossy(s.value())),
            None => f.write_str(""),
        }
    }
}
