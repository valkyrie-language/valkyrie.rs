mod display;

use crate::{
    ArgumentKey, ExpressionKind, ExpressionNode, IdentifierNode, ModifierList, NamePathNode, StatementBlock, StatementKind,
};
use core::{
    fmt::{Debug, Formatter},
    ops::Range,
};
use deriver::From;
#[cfg(feature = "pretty-print")]
use pretty_print::{
    PrettyBuilder, PrettyPrint, PrettyProvider, PrettyTree,
    helpers::{PrettySequence, SoftBlock},
};

/// A pattern match statement block
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternsList {
    /// Branches of the pattern match
    pub branches: Vec<PatternBranch>,
    /// The range of the node
    pub span: Range<u32>,
}

/// A pattern match branch
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternBranch {
    /// The condition of the branch
    pub condition: PatternCondition,
    /// The continuation of the branch
    pub continuation: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

/// All valid branches of a pattern match statement
///
/// ```vk
/// expr
/// .match {
///     case Some(a) | Success { value: a }
///     when a > 0:
///         print("a is positive")
///     when a < 0:
///         print("a is negative")
///     type Integer | Decimal:
///         print("a is a number")
///     else:
///         print("a is not a number")
/// }
/// .catch {
///     case IoError(e):
///         print("IO Error: " + e.message)
///     else:
///         print("Unknown error")
/// }
/// ```
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PatternCondition {
    /// `case Some(a) | Success { value :a }:`
    Case(PatternCaseNode),
    /// `when a > 0:`
    When(PatternWhenNode),
    /// `type Integer | Decimal:`
    Type(PatternTypeNode),
    /// `else:`
    Else,
}

impl Debug for PatternCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Case(v) => Debug::fmt(v, f),
            Self::When(v) => Debug::fmt(v, f),
            Self::Type(v) => Debug::fmt(v, f),
            Self::Else => f.write_str("Guaranteed"),
        }
    }
}

/// `case Some(a) | Success { value :a } if a > 0:`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternCaseNode {
    /// `case bind <- Some(a)`
    pub pattern: CasePattern,
    /// `case a | b | c`
    pub guard: Option<ExpressionKind>,
    /// The range of the node
    pub span: Range<u32>,
}

/// ```vk
/// let a, b = expr
/// let (a, b) = expr
/// let [a, b, **args] = expr
/// let Named(a, b) = expr
/// let Named {a, b, ***kws} = expr
/// let Named(Struct {a: b, b}, b, **args) = expr
///
/// let i = 1;
/// let j = 1;
/// let mut i, mut j;
/// let [a, b]
/// let (a, b)
/// ```
///
/// ```vk
/// case Some(a)
///    | Success { value: a }
///    | Extractor { a, b: _, *** }
///    | [a, b: _, **arg, ***kws]
/// when a > 0
///    & a is Integer:
///     do something
/// ```
///
///
/// ```vk
/// for i in range;
/// for i, j in range;
/// for mut i, mut j in range
/// for [tuple] in
/// ```
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CasePattern {
    /// `a, 'string', number, bool`
    Symbol(Box<ArgumentKey>),
    /// `(mut a, mut b)`
    Tuple(Box<TuplePatternNode>),
    /// `{ mut a: b, mut c: d }`
    Class(Box<ClassPatternNode>),
    /// `Some(a) | Success { value: a }`
    Union(Box<UnionPatternNode>),
    /// `[a, b, **]`
    Array(Box<ArrayPatternNode>),
    /// `#macro mod id`
    Atom(Box<IdentifierPattern>),
}

/// ```vk
/// let a, b = expr
/// let (a, b) = expr
/// let [a, b, **args] = expr
/// let Named(a, b) = expr
/// let Named {a, b, ***kws} = expr
/// let Named(Struct {a: b, b}, b, **args) = expr
///
/// let i = 1;
/// let j = 1;
/// let mut i, mut j;
/// let [a, b]
/// let (a, b)
/// ```
///
/// ```vk
/// case Some(a)
///    | Success { value: a }
///    | Extractor { a, b: _, *** }
///    | [a, b: _, **arg, ***kws]
/// when a > 0
///    & a is Integer:
///     do something
/// ```
///
///
/// ```vk
/// for i in range;
/// for i, j in range;
/// for mut i, mut j in range
/// for [tuple] in
/// ```
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypePattern {
    /// `a, 'string', number, bool`
    Symbol(Box<ArgumentKey>),
    /// `(mut a, mut b)`
    Tuple(Box<TuplePatternNode>),
    /// `{ mut a: b, mut c: d }`
    Class(Box<ClassPatternNode>),
    /// `Some(a) | Success { value: a }`
    Union(Box<UnionPatternNode>),
    /// `[a, b, **]`
    Array(Box<ArrayPatternNode>),
    /// `#macro mod id`
    Atom(Box<IdentifierPattern>),
}

/// `#macro mod id`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierPattern {
    /// modifiers
    pub modifiers: ModifierList,
    /// identifiers
    pub identifier: IdentifierNode,
}

/// `case Some(a) | Success { value: a }:`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionPatternNode {
    /// `case bind <- Some(a)`
    pub bind: Option<IdentifierNode>,
    /// `case a | b | c`
    pub terms: Vec<CasePattern>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `bind := module∷Named(ref a, mut b, ..c)`
///
/// ```vk
/// match term {
///     case bind: module::Name(ref a, mut b, c) if a > 0:
///         body
/// }
/// ```
///
///
/// ```vk
/// let bind = term;
/// # Option<(?, ?, ?)>
/// let unapply = module::Name::extract(bind);
/// if unapply.is_none() {
///     break;
/// }
/// # (?, ?, ?)
/// let bind = unapply!;
/// let a = unapply!.0;
/// let b = unapply!.1;
/// let c = unapply!.2;
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuplePatternNode {
    /// `bind := ...`
    pub bind: Option<IdentifierNode>,
    /// `namespace::Name(...)`
    pub name: Option<NamePathNode>,
    /// `(ref a, mut b, ..c)`
    pub terms: Vec<CasePattern>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `a := Named { a: b, c: d }`
///
///
/// ```vk
/// match term {
///     case bind@module::Name {ref a, mut b, c} if a > 0:
///         body()
/// }
/// ```
///
///
/// ```vk
/// let bind = term;
/// # Option<(?, ?, ?)>
/// let unapply = module::Name::extract(bind);
/// if unapply.is_none() {
///     break;
/// }
/// # (?, ?, ?)
/// let bind = unapply!;
/// let a = unapply!.0;
/// let b = unapply!.1;
/// let c = unapply!.2;
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassPatternNode {
    /// `bind <- ...`
    pub bind: Option<IdentifierNode>,
    /// `case namespace::Name()`
    pub name: Option<NamePathNode>,
    /// `case (ref a, mut b)`
    pub terms: Vec<CasePattern>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `bind <- [a, b, **]`
///
/// ```vk
/// match term {
///     case bind@module::Name[ref a, mut b, c] if a > 0:
///         body()
/// }
/// ```
///
///
/// ```vk
/// let bind = term;
/// # Option<(?, ?, ?)>
/// let unapply = module::Name::extract(bind);
/// if unapply.is_none() {
///     break;
/// }
/// # (?, ?, ?)
/// let bind = unapply!;
/// let a = unapply!.0;
/// let b = unapply!.1;
/// let c = unapply!.2;
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayPatternNode {
    /// `bind <- ...`
    pub bind: Option<IdentifierNode>,
    /// `[a, b, **]`
    pub terms: Vec<CasePattern>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `Soma(a) | Success { value :a } := expr`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplicitCaseNode {
    /// `Soma(a) | Success { value :a }`
    pub pattern: CasePattern,
    /// `:=`
    pub body: ExpressionNode,
    /// The range of the node
    pub span: Range<u32>,
}

/// `when a > 0:`
///
/// ```vk
/// when a > 0:
/// case _ if a > 0:
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternWhenNode {
    /// The condition of the when
    pub guard: ExpressionKind,
    /// The range of the node
    pub span: Range<u32>,
}

/// `type Integer | Decimal:`
///
///
///
/// ```vk
/// type Integer | Decimal:
/// case x if (x is (Integer | Decimal)):
/// ```
///
///
/// ```vk
/// if x is Integer {
///
/// }
/// else if x is Decimal {
///
/// }
/// ````
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PatternTypeNode {
    /// The type of the pattern
    pub typing: ExpressionKind,
    /// `when a > 0 && a < 10`
    pub guard: Option<ExpressionKind>,
    /// The range of the node
    pub span: Range<u32>,
}
impl PatternsList {
    /// Create a new pattern block
    pub fn new(capacity: usize, span: &Range<u32>) -> Self {
        Self { branches: Vec::with_capacity(capacity), span: span.clone() }
    }
}
