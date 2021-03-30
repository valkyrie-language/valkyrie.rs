use super::*;
#[cfg(feature = "pretty-print")]
mod display;

/// `caller::<T>[a]::<U>(b)?.c(d)(e) {f}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallNode<E> {
    /// The last expression in the call chain
    pub base: ExpressionType,
    /// The rest of the call chain
    pub rest: E,
    /// The range of the node
    pub span: Range<u32>,
}
impl<E> ValkyrieNode for CallNode<E> {
    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
}

/// `a: Integer = 1`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgumentTermNode<K, V, D> {
    pub key: K,
    pub value: Option<V>,
    pub default: Option<D>,
}
