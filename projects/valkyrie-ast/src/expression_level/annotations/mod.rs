use super::*;
use crate::helper::WrapDisplay;
use alloc::sync::Arc;
use nyar_error::ForeignInterfaceError;

mod builtin;
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
    pub span: SourceSpan,
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
    fn get_range(&self) -> Range<u32> {
        self.span.get_range()
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
    pub span: SourceSpan,
}

impl PartialEq<str> for AttributeTerm {
    fn eq(&self, other: &str) -> bool {
        if cfg!(debug_assertions) {
            if other.contains(":") {
                panic!("don't use id")
            }
        }
        if !self.variant.is_empty() {
            return false;
        }
        match self.path.path.as_slice() {
            [single] => other.eq(single.name.as_ref()),
            _ => false,
        }
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

impl AttributeTerm {
    /// Get the ffi module and name form attribute
    ///
    /// ```vk
    /// #ffi("module", "name")
    /// resource class A {}
    /// ```
    pub fn get_ffi_modules(&self) -> Result<(&StringTextNode, Arc<str>), ForeignInterfaceError> {
        match self.arguments.terms.as_slice() {
            [module, name] => {
                let module = match module.value.as_text() {
                    Some(s) => s,
                    None => Err(ForeignInterfaceError::InvalidForeignModule { span: self.span.clone() })?,
                };
                let name = match name.value.as_text() {
                    Some(s) => s.text.as_str(),
                    None => Err(ForeignInterfaceError::InvalidForeignName { span: self.span.clone() })?,
                };
                Ok((module, Arc::from(name)))
            }
            _ => Err(ForeignInterfaceError::InvalidForeignModule { span: self.span.clone() })?,
        }
    }
    /// Get the ffi rename
    ///
    /// ```vk
    /// class A {
    ///     #ffi("rename")
    ///     field: u32
    /// }
    /// ```
    pub fn get_ffi_rename(&self) -> Result<&StringTextNode, ForeignInterfaceError> {
        match self.arguments.terms.as_slice() {
            [rename] => {
                let name = match rename.value.as_text() {
                    Some(s) => s,
                    None => Err(ForeignInterfaceError::InvalidForeignName { span: self.span.clone() })?,
                };
                Ok(name)
            }
            _ => Err(ForeignInterfaceError::InvalidForeignModule { span: self.span.clone() })?,
        }
    }
}

impl AttributeList {
    /// Create a new modifier list.
    pub fn new(capacity: usize) -> Self {
        Self { terms: Vec::with_capacity(capacity) }
    }

    /// Check if the modifier is present.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
    /// Get the attribute by name.
    pub fn get<T>(&self, attribute: &T) -> Option<&AttributeTerm>
    where
        T: ?Sized,
        AttributeTerm: PartialEq<T>,
    {
        self.terms.iter().filter(|&x| x.eq(attribute)).next()
    }
}

impl ModifierList {
    /// Check if the modifier is present.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }

    /// Check if the modifier is present.
    pub fn contains(&self, modifier: &str) -> bool {
        self.terms.iter().any(|x| modifier.eq(x.name.as_ref()))
    }
}
