use super::{HirExpr, HirIdentifier};
use crate::{Identifier, SourceSpan};
use std::cmp::Ordering;

/// Access level for HIR items.
///
/// This enum represents the four access levels in Valkyrie:
/// - `Public`: Visible to all code
/// - `Protected`: Visible to the current class and its subclasses
/// - `Internal`: Visible within the current module
/// - `Private`: Visible only within the current class/file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccessLevel {
    /// Public access - visible to all code.
    ///
    /// Items declared with `public` modifier are accessible from any module or class.
    /// This is the most permissive access level.
    Public,

    /// Protected access - visible to current class and subclasses.
    ///
    /// Items declared with `protected` modifier are accessible within:
    /// - The class that defines them
    /// - All subclasses (direct and indirect)
    ///
    /// This is useful for implementation details that should be hidden
    /// from external code but accessible to subclasses.
    Protected,

    /// Internal access - visible within the current module.
    ///
    /// Items declared with `internal` modifier are accessible from any code
    /// within the same module, but not from other modules.
    ///
    /// This is useful for module-internal APIs that should not be exposed
    /// to external consumers.
    Internal,

    /// Private access - visible only within the current class/file.
    ///
    /// Items declared with `private` modifier are accessible only within:
    /// - The class that defines them (for class members)
    /// - The current file (for top-level items)
    ///
    /// This is the most restrictive access level.
    #[default]
    Private,
}

impl AccessLevel {
    fn rank(&self) -> u8 {
        match self {
            AccessLevel::Private => 0,
            AccessLevel::Internal => 1,
            AccessLevel::Protected => 2,
            AccessLevel::Public => 3,
        }
    }

    /// Returns true if this is public access.
    pub fn is_public(&self) -> bool {
        matches!(self, AccessLevel::Public)
    }

    /// Returns true if this is protected access.
    pub fn is_protected(&self) -> bool {
        matches!(self, AccessLevel::Protected)
    }

    /// Returns true if this is internal access.
    pub fn is_internal(&self) -> bool {
        matches!(self, AccessLevel::Internal)
    }

    /// Returns true if this is private access.
    pub fn is_private(&self) -> bool {
        matches!(self, AccessLevel::Private)
    }

    /// Returns the access level as a string for display purposes.
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessLevel::Public => "public",
            AccessLevel::Protected => "protected",
            AccessLevel::Internal => "internal",
            AccessLevel::Private => "private",
        }
    }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccessLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

/// Visibility of HIR items.
///
/// This struct combines the access level with additional visibility modifiers
/// like `readonly` for fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirVisibility {
    /// The access level of the item.
    pub access: AccessLevel,
}

impl HirVisibility {
    /// Creates a new visibility with the given access level.
    pub fn new(access: AccessLevel) -> Self {
        Self { access }
    }

    /// Creates a public visibility.
    pub fn public() -> Self {
        Self { access: AccessLevel::Public }
    }

    /// Creates a protected visibility.
    pub fn protected() -> Self {
        Self { access: AccessLevel::Protected }
    }

    /// Creates an internal visibility.
    pub fn internal() -> Self {
        Self { access: AccessLevel::Internal }
    }

    /// Creates a private visibility.
    pub fn private() -> Self {
        Self { access: AccessLevel::Private }
    }

    /// Returns true if the item is public.
    pub fn is_public(&self) -> bool {
        self.access.is_public()
    }

    /// Returns true if the item is protected.
    pub fn is_protected(&self) -> bool {
        self.access.is_protected()
    }

    /// Returns true if the item is internal.
    pub fn is_internal(&self) -> bool {
        self.access.is_internal()
    }

    /// Returns true if the item is private.
    pub fn is_private(&self) -> bool {
        self.access.is_private()
    }

    /// Returns the access level.
    pub fn access_level(&self) -> AccessLevel {
        self.access
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirKind {
    Type,
    Function(Box<HirKind>, Box<HirKind>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirType {
    /// The zero type in ADT model
    Void,
    /// The one type in ADT model
    Unit,
    Boolean,
    Integer32,
    Integer64,
    Float32,
    Float64,
    Utf8,
    Utf16,
    Infer,
    Named(Identifier),
    Apply(Box<HirType>, Vec<HirType>),
    Generic {
        name: Identifier,
        kind: HirKind,
        bounds: Vec<Identifier>,
    },
    SelfType,
    Function {
        params: Vec<HirType>,
        return_type: Box<HirType>,
    },
    Tuple(Vec<HirType>),
    Array(Box<HirType>),
    TypeLambda {
        params: Vec<HirGeneric>,
        body: Box<HirType>,
    },
    /// A trait object type representing dynamic dispatch.
    ///
    /// Trait objects use witness tables for dynamic method dispatch.
    /// At runtime, a trait object is represented as a fat pointer:
    /// - data pointer: points to the actual data
    /// - witness table pointer: points to the vtable for the trait implementation
    TraitObject {
        /// The trait being implemented.
        trait_path: Identifier,
        /// Type parameters for the trait (if any).
        type_args: Vec<HirType>,
    },
    /// An associated type projection like `Self::Item` or `<T as Trait>::Item`.
    ///
    /// This represents accessing an associated type from a trait implementation.
    /// For Generic Associated Types (GATs), the `type_args` field contains the
    /// type arguments passed to the GAT's type parameters.
    ///
    /// # Examples
    ///
    /// Simple associated type:
    /// ```v
    /// trait Iterator {
    ///     type Item
    ///     micro next(self) -> Self::Item?
    /// }
    ///
    /// micro process<T: Iterator>(iter: T) -> T::Item {
    ///     iter.next()?
    /// }
    /// ```
    ///
    /// Generic Associated Type (GAT):
    /// ```v
    /// trait LendingIterator {
    ///     type Item<'a> where Self: 'a
    ///     micro next<'a>(&'a mut self) -> Option<Self::Item<'a>>
    /// }
    ///
    /// // Here Self::Item<'a> would have type_args = [HirType::Lifetime("'a")]
    /// ```
    AssociatedType {
        /// The base type (e.g., `Self` or a type parameter).
        base: Box<HirType>,
        /// The name of the associated type.
        name: Identifier,
        /// Type arguments for Generic Associated Types (GATs).
        ///
        /// For example, in `T::Item<'a>`, this would contain the lifetime
        /// argument `'a`. For non-GAT associated types, this is empty.
        type_args: Vec<HirType>,
    },
}

impl Default for HirType {
    fn default() -> Self {
        HirType::Unit
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirGeneric {
    pub name: Identifier,
    pub kind: HirKind,
    pub bounds: Vec<Identifier>,
}

/// An associated type implementation in an impl block.
///
/// When implementing a trait with associated types, the concrete type
/// for each associated type must be specified in the impl block.
///
/// # Example
///
/// ```v
/// impl Iterator for Counter {
///     type Item = i32
///
///     micro next(self) -> Self::Item? {
///         // implementation
///     }
/// }
/// ```
///
/// In this example, `HirAssociatedTypeImpl` would represent the
/// `type Item = i32` declaration within the impl block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirAssociatedTypeImpl {
    /// The name of the associated type being implemented.
    pub name: Identifier,
    /// The concrete type that the associated type is bound to.
    pub concrete_type: HirType,
    /// Type arguments for Generic Associated Types (GATs).
    ///
    /// For example, in `type Item<'a> = &'a T`, this would contain the
    /// lifetime argument `'a`. For non-GAT associated types, this is empty.
    pub type_args: Vec<HirType>,
    /// Source span for error reporting.
    pub span: SourceSpan,
}

impl HirAssociatedTypeImpl {
    /// Creates a new associated type implementation.
    pub fn new(name: Identifier, concrete_type: HirType, span: SourceSpan) -> Self {
        Self { name, concrete_type, type_args: Vec::new(), span }
    }

    /// Creates a GAT implementation with type arguments.
    pub fn with_type_args(name: Identifier, concrete_type: HirType, type_args: Vec<HirType>, span: SourceSpan) -> Self {
        Self { name, concrete_type, type_args, span }
    }

    /// Returns true if this is a GAT implementation with type arguments.
    pub fn is_gat_impl(&self) -> bool {
        !self.type_args.is_empty()
    }
}

/// An associated constant implementation in an impl block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirAssociatedConstImpl {
    /// The name of the associated constant being implemented.
    pub name: Identifier,
    /// Optional explicit constant type annotation.
    pub const_type: Option<HirType>,
    /// The concrete value assigned to the associated constant.
    pub value: HirExpr,
    /// Source span for error reporting.
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirParam {
    pub name: HirIdentifier,
    pub ty: HirType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirPattern {
    Wildcard,
    Variable(HirIdentifier),
    Tuple(Vec<HirPattern>),
    Literal(HirLiteral),
    Constructor { name: Identifier, fields: Vec<HirPattern> },
    Or(Vec<HirPattern>),
    Type(crate::NamePath),
    Object { name: crate::NamePath, fields: Vec<(Identifier, HirPattern)> },
    Else,
}

/// HIR 层字符串片段
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirStringSegment {
    /// 文本内容
    Text(String),
    /// 插值表达式
    Interpolation {
        /// 插值表达式
        expr: HirExpr,
        /// 是否为 Fluent 变量
        is_fluent: bool,
    },
}

/// HIR 层字符串字面量
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirStringLiteral {
    /// DSL 前缀
    pub prefix: Option<Identifier>,
    /// 引号数量
    pub quote_count: u8,
    /// 字符串片段
    pub segments: Vec<HirStringSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HirLiteral {
    Integer64(i64),
    Float64(ordered_float::OrderedFloat<f64>),
    String(HirStringLiteral),
    Bool(bool),
    Unit,
}
