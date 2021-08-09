use super::*;
use alloc::sync::Arc;
use nyar_error::{SourceSpan, Validation};

mod display;

/// Pure text of a string literal.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringTextNode {
    /// The unescaped text of the string.
    pub text: String,
    /// The range of the node
    pub span: SourceSpan,
}

/// `handler"text"`, a string literal with a handler.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringLiteralNode {
    /// The raw string of the number.
    pub literal: StringTextNode,
    /// The unit of the number, if any.
    pub handler: Option<IdentifierNode>,
}

impl ValkyrieNode for StringTextNode {
    fn get_range(&self) -> Range<u32> {
        self.span.get_range()
    }
}
impl ValkyrieNode for StringLiteralNode {
    fn get_range(&self) -> Range<u32> {
        match &self.handler {
            Some(s) => s.span.get_range(),
            None => self.literal.get_range(),
        }
    }
}

impl StringTextNode {
    /// Convert to an identifier
    pub fn as_identifier(&self) -> IdentifierNode {
        IdentifierNode { name: Arc::from(self.text.as_str()), span: SourceSpan::default().with_range(self.get_range()) }
    }
}

impl StringLiteralNode {
    /// Convert to a raw string
    pub fn as_raw(&self) -> StringTextNode {
        self.literal.clone()
    }

    /// Attack a handler to the unit of the number.
    pub fn with_handler(self, handler: IdentifierNode) -> Self {
        Self { handler: Some(handler), ..self }
    }
}
