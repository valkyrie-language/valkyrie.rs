use crate::{helpers::ProgramState, StringInterpolationsNode};
use nyar_error::{Failure, NyarError, Result, SourceID, Success, SyntaxError, Validation};
use std::{mem::take, ops::Range, str::FromStr};
use valkyrie_ast::{helper::StringInterpreter, FormatterNode, FormatterTerm, StringTextNode};

/// Build a formatted string
///
///
/// ```v
/// """
/// abc
/// \n\r\u{123}
/// {a}
/// {b:fmt}
/// {c, a: f}
/// """
/// ```
pub struct StringFormatterBuilder {
    file: SourceID,
    buffer: StringTextNode,
    terms: Vec<FormatterTerm>,
    errors: Vec<NyarError>,
}

impl StringFormatterBuilder {
    /// Create a new string formatter builder
    pub fn new(file: SourceID) -> Self {
        Self { file, buffer: Default::default(), terms: vec![], errors: vec![] }
    }
    fn extend_buffer(&mut self, range: &Range<u32>) {
        let mut new = self.buffer.span.get_range();
        if self.buffer.text.is_empty() {
            new.start = range.start;
        }
        new.end = range.end;
        self.buffer.span.set_range(new)
    }
    fn push_buffer(&mut self) {
        if !self.buffer.text.is_empty() {
            let unescaped = take(&mut self.buffer);
            self.terms.push(FormatterTerm::Text { unescaped });
        }
    }
}

impl StringInterpreter for StringFormatterBuilder {
    type Output = FormatterNode;

    fn interpret(&mut self, text: &StringTextNode) -> Validation<Self::Output> {
        match StringInterpolationsNode::from_str(&text.text) {
            Ok(o) => {
                let value = o.build(self);
                Success { value, diagnostics: take(&mut self.errors) }
            }
            Err(e) => Failure { fatal: NyarError::from(e).with_file(self.file), diagnostics: take(&mut self.errors) },
        }
    }
}

impl crate::StringInterpolationsNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> FormatterNode {
        let mut list = FormatterNode::new(self.string_interpolation_term.len(), &self.span);
        for x in self.string_interpolation_term.iter() {
            if let Err(e) = x.build(ctx) {
                ctx.errors.push(e)
            }
        }
        ctx.push_buffer();
        list.terms = take(&mut ctx.terms);
        list
    }
}

impl crate::StringInterpolationTermNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        match self {
            Self::EscapeCharacter(v) => v.build(ctx),
            Self::EscapeUnicode(v) => v.build(ctx),
            Self::StringInterpolationSimple(v) => v.build(ctx),
            Self::StringInterpolationComplex(v) => v.build(ctx),
            Self::StringInterpolationText(v) => v.build(ctx),
        }
    }
}
impl crate::StringInterpolationTextNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        ctx.extend_buffer(&self.span);
        ctx.buffer.text.push_str(&self.text);
        Ok(())
    }
}

impl crate::EscapeCharacterNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        match self.text.chars().last() {
            Some(c) => {
                ctx.extend_buffer(&self.span);
                match c {
                    'n' => ctx.buffer.text.push('\n'),
                    'r' => ctx.buffer.text.push('\r'),
                    s => ctx.buffer.text.push(s),
                }
                Ok(())
            }
            None => Err(SyntaxError::new("Invalid escape sequence").with_span(ctx.file.with_range(self.span.clone())))?,
        }
    }
}

impl crate::EscapeUnicodeNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        match u32::from_str_radix(&self.code.text, 16) {
            Ok(o) => match char::from_u32(o) {
                Some(s) => {
                    ctx.extend_buffer(&self.span);
                    ctx.buffer.text.push(s);
                    Ok(())
                }
                None => Err(SyntaxError::new("unicode codepoint out of range")
                    .with_hint("The valid range is from `\\u{000000}` to `\\u{10FFFF}`")
                    .with_span(ctx.file.with_range(self.span.clone())))?,
            },
            Err(_) => Err(SyntaxError::new("invalid character found in unicode codepoint")
                .with_hint("Expect hex digits in [0-9a-fA-F]")
                .with_span(ctx.file.with_range(self.span.clone())))?,
        }
    }
}

impl crate::StringInterpolationSimpleNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        ctx.push_buffer();
        let argument = self.main_expression.build(&mut ProgramState::new(ctx.file))?;
        let formatter = self.string_formatter.as_ref().map(|v| v.build(ctx));
        ctx.terms.push(FormatterTerm::Simple { argument, formatter });
        Ok(())
    }
}
impl crate::StringFormatterNode {
    fn build(&self, sb: &mut StringFormatterBuilder) -> StringTextNode {
        let span = sb.file.with_range(self.span.clone());
        StringTextNode { text: self.text.trim().to_string(), span }
    }
}
impl crate::StringInterpolationComplexNode {
    fn build(&self, ctx: &mut StringFormatterBuilder) -> Result<()> {
        ctx.push_buffer();
        self.main_expression.build(&mut ProgramState::new(ctx.file))?;
        let argument = self.main_expression.build(&mut ProgramState::new(ctx.file))?;
        let mut formatters = Vec::with_capacity(self.tuple_pair.len());
        for x in self.tuple_pair.iter() {
            match x.to_hir(&mut ProgramState::new(ctx.file)) {
                Ok(o) => formatters.push(o),
                Err(e) => ctx.errors.push(e),
            }
        }

        ctx.terms.push(FormatterTerm::Complex { argument, formatters });
        Ok(())
    }
}
