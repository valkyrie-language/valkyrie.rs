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
    /// The annotations for `new` constructor
    pub annotations: AnnotationNode,
    /// The constructed type
    pub namepath: NamePathNode,
    /// `new List<T>()`
    pub generics: Vec<GenericCallTerm>,
    /// `new Stack()`
    pub arguments: ArgumentsList,
    /// `new List<T> { ... }`
    pub body: Vec<CollectorTerm>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `{ 1: x, p: y, [a, b]: c, **list, ***dict }`
/// A valid term in a collector
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CollectorTerm {
    /// A simple item to append
    Item(ExpressionKind),
}

impl Debug for CollectorTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Item(v) => Debug::fmt(v, f),
        }
    }
}

impl ValkyrieNode for ConstructNewNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
