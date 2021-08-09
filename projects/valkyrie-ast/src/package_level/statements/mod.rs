use super::*;
use nyar_error::SourceSpan;

mod display;

/// The top level elements in script mode.
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementKind {
    /// Placeholder for when the parser fails to parse a statement.
    Nothing,
    /// The documentation node, must have acceptor underneath.
    Document(Box<DocumentationList>),
    /// The annotation list node.
    Annotation(Box<AttributeList>),
    /// The namespace declaration node.
    Namespace(Box<NamespaceDeclaration>),
    /// The import statement node.
    Import(Box<ImportStatement>),
    /// The class declaration node.
    Class(Box<ClassDeclaration>),
    /// The union declaration node.
    Union(Box<UnionDeclaration>),
    /// The enumerate declaration node.
    Enumerate(Box<FlagDeclaration>),
    /// The trait declaration node
    Trait(Box<TraitDeclaration>),
    /// The type extension node
    Extends(Box<ExtendsStatement>),
    /// The function declaration node.
    Function(Box<FunctionDeclaration>),
    /// The let bind statement node.
    Variable(Box<LetBindNode>),
    /// The guard statement node.
    Guard(Box<GuardStatement>),
    /// The while loop statement node.
    While(Box<WhileLoop>),
    /// The for loop statement node.
    For(Box<ForLoop>),
    /// The argument argument node.
    Control(Box<ControlNode>),
    /// The argument argument node.
    Expression(Box<ExpressionNode>),
}

/// The valid terms in a class body.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatementContext {}

impl StatementKind {
    /// Create a new expression node
    pub fn expression(body: ExpressionKind, span: Range<u32>) -> Self {
        Self::Expression(Box::new(ExpressionNode { omit: false, body, span: span.clone() }))
    }
    /// Create a new raw text node
    pub fn text<S: ToString>(s: S, span: SourceSpan) -> Self {
        let literal = StringTextNode { text: s.to_string(), span: span.clone() };
        Self::expression(
            ExpressionKind::Text(Box::new(literal)),
            Range { start: span.get_start() as u32, end: span.get_end() as u32 },
        )
    }
}
