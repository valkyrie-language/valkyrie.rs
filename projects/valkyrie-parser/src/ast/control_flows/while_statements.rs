use super::*;

///
/// ```v
/// while x > 0 { ... }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStatement {
    /// Optional loop label.
    pub label: Option<Identifier>,
    /// Optional condition for while-style loops.
    pub condition: Option<TermExpression>,
    /// Loop body.
    pub body: DeclarationBody,
    /// Source span of the expression.
    pub span: Range<usize>,
}

///
/// ```v
/// while let Some(x) = item { ... }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileLetStatement {
    /// Optional loop label.
    pub label: Option<Identifier>,
    /// Optional condition for while-style loops.
    pub condition: Option<TermExpression>,
    /// Loop body.
    pub body: DeclarationBody,
    /// Source span of the expression.
    pub span: Range<usize>,
}
