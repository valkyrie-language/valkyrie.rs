mod display;

use super::*;

/// A lossless representation of the number literal
///
/// `base⁂digits.decimal⁑shift_unit;`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NumberLiteralNode {
    /// base representation of numbers
    pub base: u32,
    /// representation of input digits in base
    pub integer: String,
    /// representation of input digits in base
    pub decimal: String,
    /// Representation of input precision in base
    pub shift: isize,
    /// unit of the input number
    pub unit: Option<IdentifierNode>,
    /// The range of the node
    pub span: SourceSpan,
}

impl ValkyrieNode for NumberLiteralNode {
    fn get_range(&self) -> Range<u32> {
        self.span.get_range()
    }
}
impl NumberLiteralNode {
    /// Create a new number with given base
    pub fn new(base: u32) -> Self {
        Self { base, integer: String::new(), decimal: String::new(), shift: 0, unit: None, span: SourceSpan::default() }
    }
    /// Set the file id
    pub fn set_span(&mut self, span: SourceSpan) {
        self.span = span
    }

    /// Set the digits of the number
    pub fn set_integer(&mut self, text: &str, file: SourceID, start: usize) -> Result<(), NyarError> {
        self.integer = self.make_number(text, file, start)?;
        Ok(())
    }
    /// Set the digits of the number
    pub fn set_decimal(&mut self, text: &str, file: SourceID, start: usize) -> Result<(), NyarError> {
        self.decimal = self.make_number(text, file, start)?;
        Ok(())
    }
    /// Ensure a decimal number
    pub fn set_dot(&mut self, dot: bool) {
        if self.decimal.is_empty() && dot {
            self.decimal = "0".to_string()
        }
    }
    /// Set the precision of the number
    pub fn with_shift(self, shift: isize) -> Self {
        Self { shift, ..self }
    }
    /// Set the unit of the number
    pub fn with_unit(self, unit: IdentifierNode) -> Self {
        Self { unit: Some(unit), ..self }
    }

    fn make_number(&self, input: &str, file: SourceID, start: usize) -> Result<String, NyarError> {
        let mut buffer = String::with_capacity(input.len());
        let mut delta = 0;
        for c in input.chars() {
            match c {
                '_' => {
                    delta += 1;
                    continue;
                }
                s => {
                    if s.is_digit(self.base) {
                        delta += c.len_utf8();
                        buffer.push(s);
                    }
                    else {
                        let error = SyntaxError {
                            info: "Invalid number literal".to_string(),
                            hint: format!("invalid text `{}` in base {}", s, self.base),
                            span: {
                                let start = start + delta;
                                let end = start + delta + c.len_utf8();
                                file.with_range(start as u32..end as u32)
                            },
                        };
                        Err(error.as_error(ReportKind::Error))?
                    }
                }
            }
        }
        Ok(buffer)
    }
}
