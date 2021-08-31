use super::*;

impl<'i> IntoIterator for &'i SemanticNumber {
    type Item = &'i FlagTerm;
    type IntoIter = core::slice::Iter<'i, FlagTerm>;

    fn into_iter(self) -> Self::IntoIter {
        self.body.iter()
    }
}
