use super::*;
#[cfg(feature = "pretty-print")]
mod display;

/// `new stack Type⦓G⦔(args) { body }`
///
/// ```vk
/// new stack Type<G>(**args) {
///     value,
///     pair(key, value),
///     key: value,
///     [1]: value,
///     [1, 2:3]: body,
/// }
/// ```
///
/// ```vk
/// let body = new stack Type<G>(*args);
/// body.collect(value);
/// body.collect(pair(key, value));
/// body.key = value; # call setter
/// body[1] = value;  # call setter
/// body[1, 2:3] = value; # call setter
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConstructNewNode {
    /// The modifiers for new
    pub modifiers: Vec<IdentifierNode>,
    /// The constructed type
    pub namepath: NamePathNode,
    /// The construct generic
    pub generic: GenericCallNode,
    /// The constructor arguments
    pub arguments: ApplyCallNode,
    /// The builder arguments
    pub body: CollectorNode,
    /// The range of the node
    pub span: Range<u32>,
}

/// `{ 1: x, p: y, [a, b]: c, **list, ***dict }`
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectorNode {
    /// The collect terms
    pub terms: Vec<CollectorTerm>,
    /// The range of the node
    pub span: Range<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CollectorTerm {
    /// A simple item to append
    Item(ExpressionType),
}

impl ValkyrieNode for ConstructNewNode {
    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
}
