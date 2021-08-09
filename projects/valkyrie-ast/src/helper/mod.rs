#![doc = include_str!("readme.md")]

use core::{
    fmt::{Debug, Display, Formatter},
    ops::Range,
};

use crate::{IdentifierNode, NumberLiteralNode, StringTextNode};
#[cfg(feature = "lispify")]
pub use lispify::{Lisp, Lispify};
use nyar_error::{NyarError, Validation};
#[cfg(feature = "pretty-print")]
pub use pretty_print::{PrettyPrint, PrettyProvider, PrettyTree};

/// A node in the AST
pub trait ValkyrieNode {
    /// The range of the node
    fn get_range(&self) -> Range<u32>;
    // fn mut_range(&mut self) -> &mut Range<u32>;
    /// Get the start of the node
    fn get_start(&self) -> u32 {
        self.get_range().start
    }
    /// Get the end of the node
    fn get_end(&self) -> u32 {
        self.get_range().end
    }
}

/// A string interpreter
pub trait StringInterpreter {
    /// The output type of the interpreter
    type Output;
    /// Interpret the string
    fn interpret(&mut self, text: &StringTextNode) -> Validation<Self::Output>;
}

/// A string interpreter
pub trait NumberInterpreter {
    /// The output type of the interpreter
    type Output;
    /// Interpret the string
    fn interpret(&mut self, n: &NumberLiteralNode) -> Result<Self::Output, NyarError>;
}

/// Show display form in debug
pub struct WrapDisplay<'a, T> {
    inner: &'a T,
}
impl<'a, T> WrapDisplay<'a, T> {
    pub fn new(wrap: &'a T) -> Self {
        Self { inner: wrap }
    }
}

impl<'a, T: Display> Debug for WrapDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.inner, f)
    }
}

pub(crate) struct IdentifiersDisplay<'i> {
    inner: &'i [IdentifierNode],
}

impl<'i> IdentifiersDisplay<'i> {
    pub fn new(identifiers: &'i [IdentifierNode]) -> Self {
        Self { inner: identifiers }
    }
}
impl<'i> Debug for IdentifiersDisplay<'i> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (index, id) in self.inner.iter().enumerate() {
            if index != 0 {
                f.write_str("∷")?;
            }
            f.write_str(&id.name)?
        }
        Ok(())
    }
}
