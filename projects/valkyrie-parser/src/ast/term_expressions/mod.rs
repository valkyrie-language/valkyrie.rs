use super::*;
use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DereferenceKind {
    /// `expr.◇`
    ReadOnly,
    /// `expr.◆`
    Mutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptKind {
    /// `a[1]`/ `a[-1]` 这类序数访问。
    Ordinal,
    /// `a⁅0⁆` / `a::[0]` 这类基数访问。
    Cardinal,
}

///
///
/// ```v
/// tensor[]
/// tensor[:,:]
/// tensor[1]
/// tensor[1:-1:1]
/// tensor[1:-1:1,-1:1:-1]
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermSubscriptExpression {
    /// Whether the subscript uses ordinal or offset semantics.
    pub kind: SubscriptKind,
    /// Basement expression.
    pub base: TermExpression,
    /// Index expression.
    pub subscripts: Vec<SubscriptItem>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDereferenceExpression {
    /// 被解引用的基础表达式。
    pub base: TermExpression,
    /// 解引用权限。
    pub kind: DereferenceKind,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptItem {
    /// ```v
    /// tensor[1]
    /// ```
    Index {
        term: TermExpression,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// ```v
    /// tensor[:]
    /// tensor[::]
    /// tensor[1:-1:1]
    /// ```
    Slice {
        start: Option<TermExpression>,
        end: Option<TermExpression>,
        step: Option<TermExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
}

/// Parsed term expression node.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TermExpression {
    /// Name/path reference such as `std::console::write_line`.
    Name {
        /// Referenced path.
        path: NamePath,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// Literal value.
    Literal {
        /// Literal payload.
        literal: LiteralExpression,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// Prefix unary operator.
    Unary(Box<TermUnaryExpression>),
    /// Binary operator parsed by Pratt.
    Binary(Box<TermBinaryExpression>),
    /// Function or constructor call.
    Call(Box<TermCallExpression>),
    /// Member access such as `obj.field`.
    DotCall(Box<TermDotExpression>),
    /// Dereference access such as `ptr.◇` or `ptr.◆`.
    Dereference(Box<TermDereferenceExpression>),
    /// Subscript expression such as `items[i,j,k]`.
    Subscript(Box<TermSubscriptExpression>),
    /// Tuple literal or grouped multi-expression sequence.
    Tuple {
        /// Tuple items.
        items: Vec<TermExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// Array literal.
    Array {
        /// Array items.
        items: Vec<TermExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// Explicit cast expression such as `value as i32`.
    As(Box<TermAsExpression>),
    /// Shape assert expression such as `value is {x: i32}`.
    Is(Box<TermIsExpression>),
    /// Explicit generic application such as `value::<i32>`.
    Turbofish {
        /// Input expression.
        expr: Box<TermExpression>,
        /// Generic arguments.
        arguments: Vec<TypeExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// Assignment expression such as `self.field = value`.
    Assign {
        /// Target expression.
        target: Box<TermExpression>,
        /// Assigned value.
        value: Box<TermExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// `raise effect`
    Raise {
        /// Raised effect payload.
        value: Box<TermExpression>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// `if condition { then } else { else }`
    If(Box<IfStatement>),
    /// `if condition { then } else { else }`
    IfLet(Box<IfLetStatement>),
    /// `loop { ... }`
    Loop(Box<LoopStatement>),
    /// `loop pattern in source { ... }`
    LoopIn(Box<LoopInStatement>),
    /// `while { ... }` 或 `loop pattern in source { ... }`
    While(Box<WhileStatement>),
    /// `while { ... }` 或 `loop pattern in source { ... }`
    WhileLet(Box<WhileLetStatement>),
    /// `until { ... }` 或 `loop pattern in source { ... }`
    Until(Box<UntilStatement>),
    /// `until { ... }` 或 `loop pattern in source { ... }`
    UntilNot(Box<UntilNotStatement>),
    /// `match scrutinee { case Pattern(binding): body default: body }`
    Match {
        /// 被匹配的表达式。
        scrutinee: Box<TermExpression>,
        /// 匹配分支列表。
        arms: Vec<ArmStatement>,
        /// 源码范围。
        span: Range<usize>,
    },
    /// `catch expr { ... }`
    Catch {
        /// Protected expression.
        expr: Box<TermExpression>,
        /// Handler arms.
        arms: Vec<ArmStatement>,
        /// Source span of the expression.
        span: Range<usize>,
    },
    /// 结构体构造表达式 `Type { field: value, ... }`。
    Construct {
        /// 类型路径。
        path: NamePath,
        /// 字段初始化列表（字段名, 字段值）。
        fields: Vec<(String, TermExpression)>,
        /// 源码范围。
        span: Range<usize>,
    },
    /// Lambda 表达式 `micro(params) -> return_type { body }`。
    Lambda {
        /// 参数列表。
        params: Vec<FunctionParameter>,
        /// 可选返回类型。
        return_type: Option<TypeExpression>,
        /// 函数体。
        body: Box<DeclarationBody>,
        /// 源码范围。
        span: Range<usize>,
    },
    /// 块表达式 `unsafe { ... }` 或 `{ ... }`，内含语句序列和可选尾表达式。
    Block {
        /// 块体。
        body: Box<DeclarationBody>,
        /// 是否为 `unsafe` 块。
        is_unsafe: bool,
        /// 源码范围。
        span: Range<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermUnaryExpression {
    /// Operator.
    pub operator: UnaryOperator,
    /// Operand.
    pub base: TermExpression,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermBinaryExpression {
    /// Operator.
    pub operator: BinaryOperator,
    /// Left operand.
    pub lhs: TermExpression,
    /// Right operand.
    pub rhs: TermExpression,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermAsExpression {
    /// Input expression.
    pub base: TermExpression,
    /// Target type.
    pub target: TypeExpression,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermIsExpression {
    /// Input expression.
    pub base: TermExpression,
    /// Target type.
    pub target: PatternExpression,
    /// Source span of the expression.
    pub span: Range<usize>,
}

///
/// ```v
/// x.field
/// x.module::Class
/// x?.method::<T>() {}
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDotExpression {
    /// Object expression.
    pub base: TermExpression,
    /// Member name.
    pub caller: NamePath,
    pub arguments: TermArguments,
    /// Source span of the expression.
    pub span: Range<usize>,
}

///
///
/// ```v
/// field::<T>() { }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermCallExpression {
    /// Callee expression.
    pub callee: TermExpression,
    /// Call arguments.
    pub args: TermArguments,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermArguments {
    pub terms: Vec<TermExpression>,
}

impl TermExpression {
    /// Returns the source span of the expression.
    pub fn span(&self) -> &Range<usize> {
        match self {
            Self::Name { span, .. }
            | Self::Literal { span, .. }
            | Self::Unary(box TermUnaryExpression { span, .. })
            | Self::Binary(box TermBinaryExpression { span, .. })
            | Self::Call(box TermCallExpression { span, .. })
            | Self::DotCall(box TermDotExpression { span, .. })
            | Self::Dereference(box TermDereferenceExpression { span, .. })
            | Self::Subscript(box TermSubscriptExpression { span, .. })
            | Self::Tuple { span, .. }
            | Self::Array { span, .. }
            | Self::As(box TermAsExpression { span, .. })
            | Self::Is(box TermIsExpression { span, .. })
            | Self::Turbofish { span, .. }
            | Self::Assign { span, .. }
            | Self::Raise { span, .. }
            | Self::Catch { span, .. }
            | Self::If(box IfStatement { span, .. })
            | Self::IfLet(box IfLetStatement { span, .. })
            | Self::Loop(box LoopStatement { span, .. })
            | Self::LoopIn(box LoopInStatement { span, .. })
            | Self::While(box WhileStatement { span, .. })
            | Self::WhileLet(box WhileLetStatement { span, .. })
            | Self::Until(box UntilStatement { span, .. })
            | Self::UntilNot(box UntilNotStatement { span, .. })
            | Self::Match { span, .. }
            | Self::Construct { span, .. }
            | Self::Lambda { span, .. }
            | Self::Block { span, .. } => span,
        }
    }
}
