use super::*;
use crate::helper::IdentifiersDisplay;
use alloc::rc::Rc;
use nyar_error::SourceSpan;

mod display;
mod iters;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportKind {
    /// `import!`, share import item in all files under same namespace
    Shared,
    /// `import`, only usable in this file
    Private,
    /// `import* `, enable only under the debug mode
    Test,
}

/// `import package::module::path`
/// `import "external"`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportStatement {
    /// The annotation of the import
    pub annotation: AttributeList,
    /// The important kind
    pub kind: ImportKind,
    /// The term of the import
    pub term: ImportTermNode,
    /// The range of the node
    pub span: SourceSpan,
}

/// A valid import term of the import statement
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportTermNode {
    /// `a.b.c.{ ... }`
    Group(ImportGroupNode),
    /// `a.b.c.*`
    All(ImportAllNode),
    /// `a.b.c as alias`
    Alias(ImportAliasNode),
}

/// `group.path { item1, item2 }`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportGroupNode {
    /// The path of the import
    pub path: Vec<IdentifierNode>,
    /// The group of the import
    pub terms: Vec<ImportTermNode>,
}

/// `import package∷module.*`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAllNode {
    /// The path of the import
    pub path: Vec<IdentifierNode>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `path as alias`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportAliasNode {
    /// The path of the import
    pub path: Vec<IdentifierNode>,
    /// The item of the import
    pub item: ImportAliasItem,
    /// The alias of the import
    pub alias: Option<ImportAliasItem>,
    /// The range of the node
    pub span: Range<u32>,
}

/// The name of import items
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportAliasItem {
    /// `#attribute`
    Attribute(IdentifierNode),
    /// `@procedural`
    Procedural(IdentifierNode),
    /// `name`
    Normal(IdentifierNode),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ImportResolvedKind {
    /// `import { }`
    Empty,
    /// `import package.module.*`
    All,
    /// `import package.module.self`
    This,
    /// `import package.module.path.item as alias`
    Alias {
        /// The import item
        item: ImportAliasItem,
        /// The import item alias
        name: Option<ImportAliasItem>,
    },
}

/// A resolved import item
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImportResolvedItem {
    /// The path of the import
    pub path: Vec<Arc<str>>,
    /// The alias of the import
    pub kind: ImportResolvedKind,
    /// The position fo the resolved item
    pub span: SourceSpan,
}

/// The resolve result of import
///
/// - `function`, `constant`, `structure`, `interfaces`, `type` share a namespace
/// - `derive` and `macro` share a namespace
/// - If a macro is an implicit macro, it occupies the namespace of the function
///
/// ```vk
/// namespace a {
///     class A;
///     class B;
///     class C;
///     class D;
/// }
///
/// namespace b {
///     class A;
///     class B;
///     class C;
/// }
///
/// using a.*;
/// using b.*;
/// using a.{A, B};
/// using b.B;
/// ```
///
/// ## Script Mode
///
/// In script mode, adjusting the import statement orders at the same level does not affect the final result.
/// - You cannot explicitly import two objects with the same name, it is a compile-time error
/// - You cannot implicitly import two objects with the same name, this is a compile-time warning
/// - Extensions cannot be implicitly imported
///
/// - A: `a::A (explicit)`
/// - B: `null (duplicate, error, not available)`
/// - C: `null (ambiguous, waring, not available)`
/// - D: `a::D (implicit)`
///
/// ## Interactive Mode
///
/// In repl mode, imports always available.
/// - later implicit imports will override earlier implicit imports
/// - later explicit imports will override earlier explicit imports
///
/// - A: `a::A (explicit)`
/// - B: `b::B (explicit)`
/// - C: `b::C (implicit)`
/// - D: `a::D (implicit)`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ImportState {
    /// Available by explicit import
    Explicit,
    /// Available by implicit import
    Implicit,
    /// Unavailable due to duplicate import
    Duplicate,
    /// Unavailable due to ambiguous import
    Ambiguous,
}

impl ImportResolvedItem {
    pub fn extends(&self, path: &[IdentifierNode]) -> Self {
        let mut new = self.clone();
        new.path.extend(path.iter().map(|s| s.name.clone()));
        new
    }

    /// Join a path to the import
    fn join_group(&self, group: &ImportGroupNode, items: &mut Vec<ImportResolvedItem>) {
        let resolved = self.extends(&group.path);
        for term in &group.terms {
            term.resolve(&resolved, items);
        }
    }
    /// Join an alias to the import
    fn join_alias(&self, alias: &ImportAliasNode, items: &mut Vec<ImportResolvedItem>) {
        let mut resolved = self.extends(&alias.path);
        resolved.span.set_range(alias.get_range());
        resolved.kind = ImportResolvedKind::Alias { item: alias.item.clone(), name: alias.alias.clone() };
        items.push(resolved)
    }
    fn join_all(&self, all: &ImportAllNode, items: &mut Vec<ImportResolvedItem>) {
        let mut resolved = self.extends(&all.path);
        resolved.span.set_range(all.get_range());
        resolved.kind = ImportResolvedKind::All;
        items.push(resolved)
    }
}

impl ImportStatement {
    /// Create a new import statement
    pub fn flatten(&self) -> Vec<ImportResolvedItem> {
        let root = ImportResolvedItem { path: vec![], kind: ImportResolvedKind::Empty, span: self.span };
        let mut all = Vec::new();
        self.term.resolve(&root, &mut all);
        all
    }
}

impl ImportTermNode {
    /// Resolve the import term
    fn resolve(&self, parent: &ImportResolvedItem, all: &mut Vec<ImportResolvedItem>) {
        match self {
            Self::Alias(alias) => parent.join_alias(alias, all),
            Self::Group(group) => parent.join_group(group, all),
            Self::All(any) => parent.clone().join_all(any, all),
        }
    }
}
