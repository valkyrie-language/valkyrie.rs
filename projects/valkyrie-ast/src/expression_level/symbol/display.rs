use super::*;
use crate::helper::IdentifiersDisplay;

impl Display for BooleanNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Debug for IdentifierNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Identifier({:?}, {:?})", self.name, self.span.get_range())
    }
}

impl Display for IdentifierNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.name.as_ref())
    }
}
impl Debug for NamePathNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&IdentifiersDisplay::new(&self.path), f)
    }
}

impl Display for NamePathNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for IdentifierNode {
    fn pretty(&self, _: &PrettyProvider) -> PrettyTree {
        PrettyTree::text(self.name.to_string())
    }
}

impl Display for OutputNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.count == 0 {
            f.write_str("%%")
        }
        else if self.count < 0 {
            write!(f, "%%{}", -self.count)
        }
        else {
            write!(f, "%{}", self.count)
        }
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for OutputNode {
    fn pretty(&self, _: &PrettyProvider) -> PrettyTree {
        PrettyTree::text(self.to_string())
    }
}
#[cfg(feature = "lispify")]
impl Lispify for OutputNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        Lisp::symbol(self.to_string())
    }
}

#[cfg(feature = "lispify")]
impl Lispify for IdentifierNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        Lisp::symbol(self.to_string())
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for LambdaSlotNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.keyword(format!("${}", self.item))
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for NamePathNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.join(self.path.clone(), "∷")
    }
}
#[cfg(feature = "lispify")]
impl Lispify for NamePathNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        Lisp::symbol(self.to_string())
    }
}

impl BooleanNode {
    /// Returns the string representation of the boolean value.
    pub fn as_str(&self) -> &'static str {
        match self.value {
            true => "true",
            false => "false",
        }
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for BooleanNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.keyword(self.as_str())
    }
}
#[cfg(feature = "lispify")]
impl Lispify for BooleanNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        Lisp::symbol(self.as_str())
    }
}
impl NullNode {
    /// Returns the string representation of the null value.
    pub fn as_str(&self) -> &'static str {
        match self.nil {
            true => "nil",
            false => "null",
        }
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for NullNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.keyword(self.as_str())
    }
}

#[cfg(feature = "lispify")]
impl Lispify for NullNode {
    type Output = Lisp;

    fn lispify(&self) -> Self::Output {
        Lisp::symbol(self.as_str())
    }
}
