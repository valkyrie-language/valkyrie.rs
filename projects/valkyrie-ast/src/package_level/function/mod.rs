use super::*;
use nyar_error::SourceSpan;

mod display;

/// `micro function(args)`, `macro procedure(args)`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FunctionKind {
    /// A function that lazy evaluate the arguments
    Macro,
    /// A function that eager evaluates the arguments
    Micro,
}

/// `micro name::<T>(self: Type, value: T) { }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionDeclaration {
    /// Keyword position of the declaration
    pub keyword: SourceSpan,
    /// The belonging and name of this function
    pub name: IdentifierNode,
    /// The range of the number.
    pub kind: FunctionKind,
    /// The annotations of this function
    pub annotations: AnnotationNode,
    /// Thy type parameters of this function
    pub generics: ParametersList,
    /// The value parameters of this function
    pub parameters: ParametersList,
    /// The return type of this function
    pub returns: FunctionReturnNode,
    /// The body of this function
    pub body: StatementBlock,
}

/// `{ a; b; c }`
///
/// - Auxiliary parsing function, not instantiable.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatementBlock {
    /// The statements of this block
    pub terms: Vec<StatementKind>,
    /// The range of the node
    pub span: Range<u32>,
}

impl StatementBlock {
    /// Create a new statement block
    pub fn new(capacity: usize, span: &Range<u32>) -> Self {
        Self { terms: Vec::with_capacity(capacity), span: span.clone() }
    }
    /// Update span by the first and last statement
    pub fn update_span(&mut self) {
        if let Some(_first) = self.terms.first() {}
        if let Some(_last) = self.terms.last() {}
    }
}

/// `fun name(): ReturnType / [EffectType]`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionReturnNode {
    /// The return type of this function
    pub typing: Option<ExpressionKind>,
    /// The perform effects of this function
    pub effect: Vec<ExpressionKind>,
}

impl StatementBlock {
    /// Check if last statement has a semicolon
    pub fn last_semicolon(&self) -> bool {
        match self.terms.last() {
            Some(StatementKind::Expression(s)) => s.omit,
            _ => false,
        }
    }
    /// Fill statements with semicolon
    pub fn fill_semicolon(&mut self) {
        todo!()
    }
}

impl FunctionReturnNode {
    /// Check weather the return type and effects are empty
    pub fn is_empty(&self) -> bool {
        self.typing.is_none() && self.effect.is_empty()
    }
}

impl FunctionDeclaration {
    /// Does the function has a return type
    pub fn has_return_type(&self) -> bool {
        self.returns.typing.is_some()
    }
    /// Does the last statement has a semicolon, or it's empty
    ///
    /// Omit return always returns `( )`
    pub fn omit_return(&self) -> bool {
        !self.body.last_semicolon()
    }
}

// impl ClassDeclare {
//     pub fn get_namepath(&self) -> Iter<'_, ValkyrieIdentifier> {
//         self.namepath.iter()
//     }
//     pub fn mut_namepath(&mut self) -> &mut Vec<ValkyrieIdentifier> {
//         &mut self.namepath
//     }
//     pub fn get_modifiers(&self) -> Iter<'_, ValkyrieIdentifier> {
//         self.modifiers.iter()
//     }
//     pub fn mut_modifiers(&mut self) -> &mut Vec<ValkyrieIdentifier> {
//         &mut self.modifiers
//     }
//     pub fn get_statement(&self) -> Iter<'_, ValkyrieASTNode> {
//         self.statements.iter()
//     }
//     pub fn mut_statement(&mut self) -> &mut Vec<ValkyrieASTNode> {
//         &mut self.statements
//     }
//     pub fn to_node(self, file: SourceID, range: &Range<usize>) -> ValkyrieASTNode {
//         ValkyrieASTKind::Class(box self).to_node(file, range)
//     }
// }
