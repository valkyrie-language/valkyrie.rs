use super::*;
use core::fmt::Debug;

mod display;

use crate::{ArgumentTerm, ExpressionNode, StringTextNode};

#[doc = include_str!("readme.md")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FormatterNode {
    /// The raw string of the number.
    pub terms: Vec<FormatterTerm>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `\r, \u{00FF}, {{, }}`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FormatterTerm {
    /// `abc \r\u{12}`
    Text {
        /// The unescaped string of the number.
        unescaped: StringTextNode,
    },
    /// `{ datetime, 'yyyy-MM-dd h:mm tt', culture: 'en-us' }`
    ///
    /// equivalent to
    ///
    /// `{ datetime.show(fmt: 'yyyy-MM-dd h:mm tt', culture: 'en-us')}`
    Simple {
        /// The raw string of the number.
        argument: ExpressionKind,
        /// The format arguments
        formatter: Option<StringTextNode>,
    },
    /// `{ datetime, fmt:yyyy-MM-dd h:mm tt, culture:en-us }`
    ///
    /// equivalent to
    ///
    /// `{ datetime.show(fmt: 'yyyy-MM-dd h:mm tt', culture: 'en-us')}`
    Complex {
        /// The raw string of the number.
        argument: ExpressionKind,
        /// The format arguments
        formatters: Vec<ArgumentTerm>,
    },
}

impl ValkyrieNode for FormatterNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

impl FormatterNode {
    /// Create a string formatter
    pub fn new(capacity: usize, span: &Range<u32>) -> Self {
        Self { terms: Vec::with_capacity(capacity), span: span.clone() }
    }
}
