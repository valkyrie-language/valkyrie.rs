mod display;
use super::*;

/// The kind of namespace
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NamespaceKind {
    /// Main namespace where definitions and imports can be shared
    Main,
    /// Independent namespace, isolated definitions, except public and main definitions
    Standalone,
    /// This is a test file, only available in the test environment
    Test,
    /// Temporarily remove a file
    Hide,
}

/// `namespace std.math`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceDeclaration {
    /// The kind of the namespace
    pub kind: NamespaceKind,
    /// The path of the namespace
    pub path: NamePathNode,
    /// The range of the node
    pub span: Range<u32>,
}

impl NamespaceDeclaration {
    /// Create a new namespace declaration
    pub fn new<I>(names: I, range: Range<u32>) -> Self
    where
        I: IntoIterator<Item = IdentifierNode>,
    {
        Self { kind: NamespaceKind::Standalone, path: names.into_iter().collect(), span: range.clone() }
    }
    /// Create a new namespace declaration
    pub fn with_kind(mut self, kind: NamespaceKind) -> Self {
        self.kind = kind;
        self
    }
}

//
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct NamespaceDeclare {
//     pub kind: NamespaceKind,
//     pub name: Vec<String>,
// }
//
// impl NamespaceDeclare {
//     pub fn new(kind: &str) -> Self {
//         Self { kind: NamespaceKind::from_str(kind).unwrap(), name: Vec::new() }
//     }
//     pub fn push_name(&mut self, name: impl Into<String>) {
//         self.name.push(name.into());
//     }
//     pub fn to_node(self, file: SourceID, range: &Range<usize>) -> ValkyrieASTNode {
//         ValkyrieASTKind::Namespace(box self).to_node(file, range)
//     }
// }
//
// impl FromStr for NamespaceKind {
//     type Err = !;
//
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let out = match s {
//             "namespace!" => Self::Shared,
//             "namespace*" => Self::Test,
//             _ => Self::Unique,
//         };
//         Ok(out)
//     }
// }
