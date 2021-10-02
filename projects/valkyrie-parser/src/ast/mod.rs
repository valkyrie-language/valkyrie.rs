mod control_flows;
mod pattern_expressions;
mod term_expressions;
mod type_expressions;

pub use self::{
    control_flows::{
        if_statements::{IfLetStatement, IfStatement},
        loop_statements::{LoopInStatement, LoopStatement},
        until_statements::{UntilNotStatement, UntilStatement},
        while_statements::{WhileLetStatement, WhileStatement},
        BreakStatement, ContinueStatement, FallthroughStatement, ResumeStatement, ReturnStatement, YieldFromStatement, YieldStatement,
    },
    pattern_expressions::{
        ArmStatement, ArrayPattern, CaseArm, ElseArm, ExtractPattern, MatchObjectField, ObjectPattern, PatternExpression, PatternOrExpression,
        TuplePattern, TypeArm,
    },
    term_expressions::{
        DereferenceKind, SubscriptItem, SubscriptKind, TermArguments, TermAsExpression, TermBinaryExpression, TermCallExpression,
        TermDereferenceExpression, TermDotExpression, TermExpression, TermIsExpression, TermSubscriptExpression, TermUnaryExpression,
    },
    type_expressions::{PointerKind, RowMethodTypeExpression, TypeExpression},
};

use std::ops::Range;
use valkyrie_types::Identifier;

/// 具备源码跨度的标识符节点。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentifierNode {
    /// 标识符语义名称。
    pub name: Identifier,
    /// 标识符在源码中的跨度。
    pub span: Range<usize>,
}

impl IdentifierNode {
    /// 创建一个新的标识符节点。
    pub fn new(name: Identifier, span: Range<usize>) -> Self {
        Self { name, span }
    }

    /// 以字符串形式访问标识符名称。
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

impl std::fmt::Display for IdentifierNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.name, f)
    }
}

/// Parser root for a single Valkyrie source file.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValkyrieRoot {
    /// Top-level declarations in source order.
    pub statements: Vec<RootStatement>,
}

/// Strongly typed top-level declaration node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RootStatement {
    /// `namespace foo::bar;`
    Namespace(NamespaceDeclaration),
    /// `using foo::bar;`
    Using(UsingStatement),
    /// Function declaration with optional body.
    Function(FunctionDeclaration),
    /// Class declaration with fields and methods.
    Class(ClassDeclaration),
    /// Trait declaration with required/default methods.
    Trait(TraitDeclaration),
    /// `imply` declaration for inherent or trait implementations.
    Imply(ImplyDeclaration),
    /// Unite declaration with closed nominal variants.
    Unite(UniteDeclaration),
    /// `attribute` declaration for defining marker attributes.
    Attribute(AttributeDeclaration),
    /// `type Name = Target;` 类型别名声明。
    TypeAlias(TypeAliasDeclaration),
}

/// `type Name = Target;` 类型别名声明节点。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypeAliasDeclaration {
    /// 别名名称。
    pub name: IdentifierNode,
    /// 别名指向的目标类型。
    pub target: TypeExpression,
    /// 源码跨度。
    pub span: Range<usize>,
}

/// `attribute <name>;` 声明节点。
///
/// 用于声明可在类、联合等类型上使用的标记属性，
/// 如 `attribute commander;` 声明一个名为 `commander` 的标记。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeDeclaration {
    /// 属性名称。
    pub name: IdentifierNode,
    /// 源码跨度。
    pub span: Range<usize>,
}

/// Namespace declaration node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamespaceDeclaration {
    /// Parsed namespace path.
    pub name: NamePath,
    /// Parsed body statements inside the namespace block.
    pub body: Option<DeclarationBody>,
    /// Source span of the declaration.
    pub span: Range<usize>,
}

/// Using/import declaration node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UsingStatement {
    /// Imported path.
    pub path: NamePath,
    /// 选择性导入的名称列表（`using! a.b.{C, D}` 语法）。
    pub selective_imports: Vec<String>,
    /// 是否为通配导入（`using a.b.*;` 语法）。
    pub glob_import: bool,
    /// Source span of the declaration.
    pub span: Range<usize>,
}

/// Reusable path node with explicit span.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamePath {
    /// Individual path components.
    pub parts: Vec<String>,
    /// Source span of the whole path.
    pub span: Range<usize>,
}

/// Named type path with generic arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypePath {
    /// Type path name.
    pub name: NamePath,
    /// Generic arguments attached to the type path.
    pub arguments: Vec<TypeExpression>,
    /// Source span of the type.
    pub span: Range<usize>,
}

/// Flat attribute argument captured by the lightweight parser.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeArgument {
    /// Optional named-argument key.
    pub key: Option<String>,
    /// Structured argument expression.
    pub value: TermExpression,
}

/// Structured attribute node such as `[clr("Type", "Assembly", "Method")]`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeItem {
    /// Parsed attribute name.
    pub name: NamePath,
    /// Parsed attribute arguments.
    pub arguments: Vec<AttributeArgument>,
    /// Source span of the attribute item.
    pub span: Range<usize>,
}

/// One bracketed attribute list such as `[main, inline]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeList {
    /// Individual attribute items within the bracket pair.
    pub items: Vec<AttributeItem>,
}

/// Common declaration annotations aligned with the C# frontend shape.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Annotations {
    /// Documentation lines attached above the declaration.
    pub documents: Vec<String>,
    /// Parsed attribute lists.
    pub attribute_lists: Vec<AttributeList>,
    /// Declaration modifiers such as `public` or `abstract`.
    pub modifiers: Vec<IdentifierNode>,
}

impl Annotations {
    /// Returns a flattened iterator over all attribute items.
    pub fn attributes(&self) -> impl Iterator<Item = &AttributeItem> {
        self.attribute_lists.iter().flat_map(|list| list.items.iter())
    }
}

/// Parsed function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterPassingKind {
    /// 默认只读引用传递。
    Ref,
    /// 可变引用传递。
    Mut,
    /// 所有权移动传递。
    Own,
}

/// Parsed function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionParameter {
    /// Parameter name.
    pub name: IdentifierNode,
    /// Structured type annotation.
    pub parameter_type: Option<TypeExpression>,
    /// Whether the parameter uses `mut`.
    pub is_mutable: bool,
    /// 参数传递方式。
    pub passing: ParameterPassingKind,
    /// Source span of the parameter text.
    pub span: Range<usize>,
}

/// Structured generic parameter captured from a declaration header.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericParameterDeclaration {
    /// Generic parameter name.
    pub name: IdentifierNode,
    /// Optional trait/type bounds attached with `:`.
    pub bounds: Vec<TypeExpression>,
    /// Optional default type attached with `=`.
    pub default_type: Option<TypeExpression>,
    /// Source span of the generic parameter fragment.
    pub span: Range<usize>,
}

/// Structured `where` constraint such as `T: Display + Clone`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhereConstraintDeclaration {
    /// The constrained type expression.
    pub target_type: TypeExpression,
    /// Bounds required by the constraint.
    pub bounds: Vec<TypeExpression>,
    /// Source span of the whole constraint.
    pub span: Range<usize>,
}

/// Parsed inheritance item.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InheritanceItem {
    /// Optional alias used for renamed inheritance.
    pub alias: Option<String>,
    /// Base class or trait type.
    pub base_type: TypeExpression,
    /// Source span of the inheritance item.
    pub span: Range<usize>,
}

/// One statement inside a declaration body.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FunctionStatement {
    /// `let value: i64 = 42;`
    Let(LetStatement),
    /// Expression statement.
    Term {
        /// Parsed expression payload.
        expression: TermExpression,
        /// Source span of the statement.
        span: Range<usize>,
    },
    /// Nested function statement.
    Function {
        /// Parsed function declaration payload.
        function: FunctionDeclaration,
        /// Source span of the statement.
        span: Range<usize>,
    },
    /// `break value`
    Break(BreakStatement),
    /// `continue`
    Continue(ContinueStatement),
    /// `yield value`
    Yield(YieldStatement),
    /// `yield from iterator`
    YieldFrom(YieldFromStatement),
    /// `return value`
    Return(ReturnStatement),
    /// `resume value`
    Resume(ResumeStatement),

    /// `fallthrough` 语句级控制流。
    ///
    /// 暂作为表达式节点承载，实际仅允许出现在 `case` statement 体系中；
    /// 在 `match` 表达式、普通块或其它上下文中应在语义阶段被拒绝。
    Fallthrough(FallthroughStatement),
}

impl FunctionStatement {
    /// Returns the source span of the statement.
    pub fn span(&self) -> &Range<usize> {
        match self {
            Self::Let(statement) => &statement.span,
            Self::Term { span, .. } | Self::Function { span, .. } => span,
            Self::Yield(statement) => &statement.span,
            Self::YieldFrom(statement) => &statement.span,
            Self::Continue(statement) => &statement.span,
            Self::Break(statement) => &statement.span,
            Self::Return(statement) => &statement.span,
            Self::Resume(statement) => &statement.span,
            Self::Fallthrough(statement) => &statement.span,
        }
    }
}

/// Parsed let binding statement.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LetStatement {
    /// Whether the binding starts with `mut`.
    pub is_mutable: bool,
    /// Binding pattern.
    pub pattern: PatternExpression,
    /// Optional type annotation.
    pub ty: Option<TypeExpression>,
    /// Optional initializer expression.
    pub initializer: Option<TermExpression>,
    /// Source span of the statement.
    pub span: Range<usize>,
}

/// Literal payload owned by the parser AST.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LiteralExpression {
    /// Integer literal text.
    Integer(String),
    /// Float literal text.
    Float(String),
    /// Structured string literal payload.
    String(StringLiteral),
    /// Boolean literal.
    Bool(bool),
    /// Unit literal `()`.
    Unit,
    /// Null pointer literal.
    Null,
}

/// Parsed string segment inside a string literal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringSegment {
    /// Plain decoded text.
    Text(String),
    /// Embedded interpolation expression.
    Interpolation {
        /// Parsed interpolation expression.
        expression: Box<TermExpression>,
        /// Whether the interpolation refers to the special fluent variable `$`.
        is_fluent: bool,
    },
}

/// Structured parser-side string literal.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringLiteral {
    /// Optional prefix such as `r`.
    pub prefix: Option<String>,
    /// Delimiter quote count, usually `1` or `3`.
    pub quote_count: u8,
    /// Structured string segments.
    pub segments: Vec<StringSegment>,
}

/// Prefix unary operators supported by the parser AST.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    /// `-value`
    Neg,
    /// `!value`
    Not,
}

/// Infix operators supported by the parser AST.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BinaryOperator {
    /// `lhs && rhs`
    And,
    /// `lhs || rhs`
    Or,
    /// `lhs + rhs`
    Add,
    /// `lhs - rhs`
    Sub,
    /// `lhs * rhs`
    Mul,
    /// `lhs / rhs`
    Div,
    /// `lhs % rhs`
    Rem,
    /// `lhs ^ rhs`，幂运算。
    Power,
    /// `lhs == rhs`
    Eq,
    /// `lhs != rhs`
    Ne,
    /// `lhs < rhs`
    Lt,
    /// `lhs <= rhs`
    Le,
    /// `lhs > rhs`
    Gt,
    /// `lhs >= rhs`
    Ge,
    /// `lhs |> rhs`，管道操作符，将左侧值传入右侧函数。
    Pipe,
    /// `lhs << rhs`，左移位运算符。
    Shl,
    /// `lhs >> rhs`，右移位运算符。
    Shr,
    /// `lhs & rhs`，按位与运算符。
    BitAnd,
    /// `lhs | rhs`，按位或运算符。
    BitOr,
}

/// Structured declaration body captured by recursive descent.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeclarationBody {
    /// Statement list in source order.
    pub statements: Vec<FunctionStatement>,
    /// Optional final expression without a trailing semicolon.
    pub tail_expression: Option<TermExpression>,
    /// Source span of the body content.
    pub span: Range<usize>,
}

/// Parsed object field declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectFieldDeclaration {
    /// Structured annotations attached above the field.
    pub annotations: Annotations,
    /// Field name.
    pub name: IdentifierNode,
    /// Structured field type annotation.
    pub field_type: TypeExpression,
    /// Optional parsed default value expression.
    pub default_value: Option<TermExpression>,
    /// Source span of the field declaration.
    pub span: Range<usize>,
}

/// Parsed object method declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectMethodDeclaration {
    /// Method name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the method.
    pub annotations: Annotations,
    /// Original signature line.
    pub signature: String,
    /// Structured parameters.
    pub params: Vec<FunctionParameter>,
    /// Optional structured return type.
    pub return_type: Option<TypeExpression>,
    /// Optional method body.
    pub body: Option<DeclarationBody>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Parsed trait associated type declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitAssociatedTypeDeclaration {
    /// Structured annotations attached above the associated type.
    pub annotations: Annotations,
    /// Associated type name.
    pub name: IdentifierNode,
    /// Raw generic parameter fragments captured from the source clause.
    pub generic_parameters: Vec<String>,
    /// Optional trait bounds such as `Display + Clone`.
    pub bounds: Vec<TypeExpression>,
    /// Optional default type expression.
    pub default_type: Option<TypeExpression>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Parsed trait associated constant declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitAssociatedConstDeclaration {
    /// Structured annotations attached above the associated constant.
    pub annotations: Annotations,
    /// Constant name.
    pub name: IdentifierNode,
    /// Structured constant type annotation.
    pub const_type: TypeExpression,
    /// Optional default value provided inside the trait.
    pub default_value: Option<TermExpression>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Parsed associated type binding inside an `imply` block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplyAssociatedTypeBinding {
    /// Structured annotations attached above the binding.
    pub annotations: Annotations,
    /// Associated type name.
    pub name: IdentifierNode,
    /// Structured generic parameters attached to the associated type binding.
    pub generic_parameters: Vec<GenericParameterDeclaration>,
    /// Concrete type expression bound to the associated type.
    pub concrete_type: TypeExpression,
    /// Source span of the whole binding.
    pub span: Range<usize>,
}

/// Parsed associated constant binding inside an `imply` block.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplyAssociatedConstBinding {
    /// Structured annotations attached above the binding.
    pub annotations: Annotations,
    /// Associated constant name.
    pub name: IdentifierNode,
    /// Optional explicit constant type annotation.
    pub const_type: Option<TypeExpression>,
    /// Concrete value expression bound to the constant.
    pub value: TermExpression,
    /// Source span of the whole binding.
    pub span: Range<usize>,
}

/// Object body used by classes and concrete trait declarations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectBody {
    /// Declared fields.
    pub fields: Vec<ObjectFieldDeclaration>,
    /// Declared methods.
    pub methods: Vec<ObjectMethodDeclaration>,
    /// Declared associated types.
    pub associated_types: Vec<TraitAssociatedTypeDeclaration>,
    /// Declared associated constants.
    pub associated_constants: Vec<TraitAssociatedConstDeclaration>,
}

/// Class declaration node aligned with the C# frontend's object model.
///
/// 同时承载 `class`（引用类型）和 `structure`（值类型）两种声明，
/// 通过 `is_value_type` 字段区分。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDeclaration {
    /// Class name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the declaration.
    pub annotations: Annotations,
    /// Optional parent list from `class Child(Base, Interface)`.
    pub inheritance: Vec<InheritanceItem>,
    /// Parsed object body.
    pub body: ObjectBody,
    /// 是否为值类型（`structure` 关键字声明）。
    ///
    /// `true` 表示值类型（`structure`），`false` 表示引用类型（`class`）。
    pub is_value_type: bool,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Trait declaration node aligned with the C# frontend's object model.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitDeclaration {
    /// Trait name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the declaration.
    pub annotations: Annotations,
    /// Raw generic parameter fragments captured from the source clause.
    pub generic_parameters: Vec<String>,
    /// Optional super-trait list from `trait X: Y, Z`.
    pub inheritance: Vec<InheritanceItem>,
    /// Optional alias targets from `trait Alias = A + B`.
    pub alias_targets: Vec<InheritanceItem>,
    /// Whether this declaration is a trait alias instead of a full body.
    pub is_alias: bool,
    /// Parsed object body containing methods and associated items.
    pub body: ObjectBody,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Parsed `imply` declaration aligned with trait witness and inherent implementation blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplyDeclaration {
    /// Structured annotations attached above the declaration.
    pub annotations: Annotations,
    /// Structured generic parameters attached to the `imply` header.
    pub generic_parameters: Vec<GenericParameterDeclaration>,
    /// Target type being extended or implementing a trait.
    pub target_type: TypeExpression,
    /// Optional trait or protocol being implemented.
    pub trait_type: Option<TypeExpression>,
    /// Structured `where` constraints attached to the `imply` header.
    pub where_constraints: Vec<WhereConstraintDeclaration>,
    /// Methods defined in the `imply` block.
    pub methods: Vec<ObjectMethodDeclaration>,
    /// Associated type bindings defined in the `imply` block.
    pub associated_type_bindings: Vec<ImplyAssociatedTypeBinding>,
    /// Associated constant bindings defined in the `imply` block.
    pub associated_const_bindings: Vec<ImplyAssociatedConstBinding>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// One declared variant inside a `unite`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UniteVariantDeclaration {
    /// Variant name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the variant.
    pub annotations: Annotations,
    /// Optional object-like field payload.
    pub fields: Vec<ObjectFieldDeclaration>,
    /// 元组变体的类型列表，如 `Some(T)` 中的 `[T]`。
    pub tuple_types: Vec<TypeExpression>,
    /// Optional GADT-style result type.
    pub result_type: Option<TypeExpression>,
    /// Source span of the whole variant declaration.
    pub span: Range<usize>,
}

/// Unite declaration node aligned with nominal sealed-family lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UniteDeclaration {
    /// Unite name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the declaration.
    pub annotations: Annotations,
    /// Declared variants.
    pub variants: Vec<UniteVariantDeclaration>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}

/// Function declaration node owned by the parser layer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionDeclaration {
    /// Function name.
    pub name: IdentifierNode,
    /// Structured annotations attached above the declaration.
    pub annotations: Annotations,
    /// Original signature line.
    pub signature: String,
    /// Structured parameters.
    pub params: Vec<FunctionParameter>,
    /// Optional structured return type.
    pub return_type: Option<TypeExpression>,
    /// Optional function body.
    pub body: Option<DeclarationBody>,
    /// Source span of the whole declaration.
    pub span: Range<usize>,
}
