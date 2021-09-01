use super::*;

mod display;

/// `let mut pattern = expression`
///
///
/// ```vk
/// let x;
/// let x: i32;
/// let x = 1;
/// let x: i32 = 1;
/// let mut x: i32 = 1;
/// let (x, ) = 1
/// let Some(x) = expr;
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LetBindNode {
    /// The annotation of the variable
    pub pattern: CasePattern,
    /// The type of the variable
    pub type_hint: Option<ExpressionKind>,
    /// The default value of the variable
    pub body: Option<ExpressionKind>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `let mut pattern = expression`
///
///
/// ```vk
/// let x;
/// let x: i32;
/// let x = 1;
/// let x: i32 = 1;
/// let mut x: i32 = 1;
/// let (x, ) = 1
/// let Some(x) = expr;
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariableDeclaration {
    /// The annotation of the variable
    pub identifier: IdentifierNode,
    /// The type of the variable
    pub type_hint: Option<ExpressionKind>,
    /// The default value of the variable
    pub body: Option<ExpressionKind>,
}

impl LetBindNode {
    pub fn canonicalization(self) -> Vec<VariableDeclaration> {
        match &self.pattern {
            CasePattern::Symbol(_) => {}
            CasePattern::Tuple(_) => {}
            CasePattern::Class(_) => {}
            CasePattern::Union(_) => {}
            CasePattern::Array(_) => {}
            CasePattern::Atom(_) => {}
        }
        todo!()
    }
}
