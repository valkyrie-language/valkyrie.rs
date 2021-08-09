use super::*;
use alloc::sync::Arc;

mod convert;
mod display;

/// A node representing a identifier.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierNode {
    /// The name of the identifier.
    pub name: Arc<str>,
    /// The location of this identifier.
    pub span: SourceSpan,
}

/// `package∷module∷name`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamePathNode {
    /// The names of the namepath.
    pub path: Vec<IdentifierNode>,
    /// The location of this namepath.
    pub span: SourceSpan,
}

/// `package∷module∷name`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BooleanNode {
    /// `true, false`
    pub value: bool,
    /// The range of the node
    pub span: Range<u32>,
}

/// `null, nil`, type of null value
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NullNode {
    /// Whether the collection is empty or does not exist
    pub nil: bool,
    /// The range of the node
    pub span: Range<u32>,
}

/// `%1, %%1`, the number of the reference
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutputNode {
    /// - positive number indicates the nth one
    /// - negative number indicates the nth one from the last
    pub count: isize,
    /// The range of the node
    pub span: Range<u32>,
}

/// `$, $1, $x`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LambdaSlotNode {
    /// The repeat times of `$`
    pub level: usize,
    /// The name of the slot
    pub item: LambdaSlotItem,
    /// The location of the slot
    pub span: Range<u32>,
}

/// `$, $0, $1, $x`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LambdaSlotItem {
    /// `$, $.call()`, Automatically obtain a number based on context
    SelfType,
    /// `$0, type_of($)`, The meta info of the slot
    MetaType,
    /// `$1, $.arguments.1`, The slot index from the lambda
    Index(NonZeroU64),
    /// `$x, $.arguments.x`, The slot name from the lambda
    Named(IdentifierNode),
}

impl NamePathNode {
    /// Create a new name path node with given identifiers.
    pub fn new<I>(names: I) -> Self
    where
        I: IntoIterator<Item = IdentifierNode>,
    {
        Self { path: names.into_iter().collect(), span: Default::default() }
    }
    /// Push a new identifier to the name path.
    pub fn join<I: IntoIterator<Item = IdentifierNode>>(mut self, other: I) -> Self {
        self.path.extend(other);
        self
    }
    /// Calculate range by first and last elements
    pub fn get_range(&self) -> Range<u32> {
        self.span.get_range()
    }
    /// Set the file span for namepath
    pub fn with_span(self, span: SourceSpan) -> Self {
        Self { span, ..self }
    }
}

impl IdentifierNode {
    /// Set the file for namepath
    pub fn with_file(mut self, file: SourceID) -> Self {
        self.span.set_file(file);
        self
    }
    /// Set the
    pub fn with_span<I>(mut self, range: Range<u32>) -> Self {
        self.span.set_range(range);
        self
    }
    /// Whether this identifier needs escaping
    pub fn is_normal(&self) -> bool {
        self.name.starts_with(|c: char| c.is_ascii_lowercase())
    }
}

// impl ValkyrieIdentifier {
//     pub fn new(name: impl Into<String>, file: SourceID, range: &Range<usize>) -> Self {
//         Self { name: name.into(), span: SourceSpan { file, head: range.start, tail: range.end } }
//     }
//     pub fn to_node(self, file: SourceID, range: &Range<usize>) -> ValkyrieASTNode {
//         ValkyrieASTKind::Namepath(vec![self]).to_node(file, range)
//     }
// }
//
// impl ValkyrieASTNode {
//     pub fn identifier(name: impl Into<String>, file: SourceID, range: &Range<usize>) -> Self {
//         ValkyrieIdentifier::new(name, file, range).to_node(file, range)
//     }
//     pub fn namepath(items: Vec<ValkyrieIdentifier>, file: SourceID, range: &Range<usize>) -> Self {
//         Self { kind: ValkyrieASTKind::Namepath(items), span: SourceSpan::new(file, range) }
//     }
// }
