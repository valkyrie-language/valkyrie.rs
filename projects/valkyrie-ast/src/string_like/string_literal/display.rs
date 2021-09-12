use super::*;

impl Display for StringTextNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_char('\'')?;
        for c in self.text.chars() {
            match c {
                '\n' => f.write_str("\\n")?,
                '\r' => f.write_str("\\r")?,
                '\t' => f.write_str("\\t")?,
                '\\' => f.write_str("\\\\")?,
                '\'' => f.write_str("\\'")?,
                _ => f.write_char(c)?,
            }
        }
        f.write_char('\'')
    }
}

impl Display for StringLiteralNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(unit) = &self.handler {
            f.write_str(&unit.name.as_ref())?;
        }
        Display::fmt(&self.literal, f)
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for StringLiteralNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.string(self.to_string())
    }
}

#[cfg(feature = "lispify")]
impl Lispify for StringLiteralNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        let literal = Lisp::string(self.literal.to_string());
        match &self.handler {
            Some(s) => Lisp::unit(s.name.clone()) & literal,
            None => literal,
        }
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for StringTextNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.string(self.to_string())
    }
}
