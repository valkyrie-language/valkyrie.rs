use super::*;
use crate::helper::WrapDisplay;

mod display;

/// `#module∷name.variant(args) { ... } modifiers`
///
/// The annotations of the statements
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnnotationNode {
    /// The documentations of the statement
    pub documents: DocumentationList,
    /// The attributes of the statement
    pub attributes: AttributeList,
    /// The modifiers of the statement
    pub modifiers: ModifierList,
}

/// `@module∷name(args) { ... }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProceduralNode {
    /// The kind of this attribute.
    pub kind: AttributeKind,
    /// The names of this attribute.
    pub path: NamePathNode,
    /// The arguments of this attribute.
    pub arguments: ArgumentsList,
    /// The capture of this attribute.
    pub domain: Option<StatementBlock>,
    /// The range of the node
    pub span: Range<u32>,
}

impl From<ProceduralNode> for AttributeTerm {
    fn from(node: ProceduralNode) -> Self {
        AttributeTerm {
            kind: node.kind,
            path: node.path,
            variant: vec![],
            arguments: node.arguments,
            domain: node.domain,
            span: node.span,
        }
    }
}

impl ValkyrieNode for ProceduralNode {
    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
}

/// A namepath is a series of identifiers separated by dots.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeKind {
    /// `#`
    Normal,
    /// `##`
    Environment,
    /// `#!`
    Script,
}

/// `@[module∷name.function(args), module∷name.function2(args)] <CAPTURE>`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeList {
    /// The modifiers in group
    pub terms: Vec<AttributeTerm>,
}

/// `module∷name.variant(args) { CAPTURE }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeTerm {
    /// The kind of this attribute.
    pub kind: AttributeKind,
    /// The names of this attribute.
    pub path: NamePathNode,
    /// The names of this attribute.
    pub variant: Vec<IdentifierNode>,
    /// The arguments of this attribute.
    pub arguments: ArgumentsList,
    /// The dsl part of the attribute
    pub domain: Option<StatementBlock>,
    /// The range of the node
    pub span: Range<u32>,
}

impl AttributeTerm {
    /// Interpreted as an external function call
    pub fn as_ffi(&self) -> Result<(&str, &str), NyarError> {
        let module = match self.arguments.terms.get(0) {
            Some(s) => match &s.value {
                ExpressionKind::Text(s) => s.text.as_str(),
                _ => {
                    panic!("1")
                }
            },
            None => {
                panic!("2")
            }
        };
        let name = match self.arguments.terms.get(1) {
            Some(s) => match &s.value {
                ExpressionKind::Text(s) => s.text.as_str(),
                _ => {
                    panic!("3")
                }
            },
            None => {
                panic!("4")
            }
        };
        Ok((module, name))
    }
}

/// `public static final synchronized class Main {}`
///
/// - Auxiliary parsing function, not instantiable.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModifierList {
    /// The modifiers in group
    pub terms: Vec<IdentifierNode>,
}

impl ModifierList {
    /// Create a new modifier list with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self { terms: Vec::with_capacity(capacity) }
    }
}

impl Default for AttributeKind {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for AnnotationNode {
    fn default() -> Self {
        Self { documents: Default::default(), attributes: Default::default(), modifiers: Default::default() }
    }
}
impl AnnotationNode {
    /// Check if the modifiers, attributes and documents are empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty() && self.attributes.is_empty() && self.modifiers.is_empty()
    }
}
impl From<AttributeList> for AnnotationNode {
    fn from(value: AttributeList) -> Self {
        Self { documents: Default::default(), attributes: value, modifiers: Default::default() }
    }
}
impl AttributeKind {
    /// Returns the string representation of the macro kind.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "#",
            Self::Environment => "##",
            Self::Script => "#!",
        }
    }
}

impl AttributeTerm {}

impl AttributeList {
    /// Create a new modifier list.
    pub fn new(capacity: usize) -> Self {
        Self { terms: Vec::with_capacity(capacity) }
    }

    /// Check if the modifier is present.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

impl ModifierList {
    /// Check if the modifier is present.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Check if the modifier is present.
    pub fn contains(&self, modifier: &str) -> bool {
        self.terms.iter().any(|x| x.name.eq(modifier))
    }
}
