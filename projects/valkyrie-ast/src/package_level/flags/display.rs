use super::*;

impl Debug for SemanticNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let w = &mut match self.kind {
            SemanticKind::Enumerate => f.debug_struct("Enumerate"),
            SemanticKind::Flags => f.debug_struct("Flags"),
        };
        if !self.annotations.is_empty() {
            w.field("annotations", &self.annotations);
        }
        w.field("name", &self.name);
        w.field("terms", &self.body);
        if let Some(inherits) = &self.layout {
            w.field("layout", inherits);
        }
        if let Some(inherits) = &self.implements {
            w.field("implements", inherits);
        }
        w.field("body", &self.body);
        w.field("span", &self.span);
        w.finish()
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for SemanticNumber {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(4);
        terms += theme.keyword("enumerate");
        terms += " ";
        terms += self.namepath.pretty(theme);
        terms += self.body.pretty(theme);
        terms.into()
    }
}

impl Debug for FlagTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Macro(v) => Debug::fmt(v, f),
            Self::Encode(v) => Debug::fmt(v, f),
            Self::Method(v) => Debug::fmt(v, f),
        }
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for EncodeDeclaration {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(3);
        terms += self.name.pretty(theme);
        if let Some(value) = &self.value {
            terms += " ";
            terms += theme.operator("=");
            terms += " ";
            terms += value.pretty(theme);
        }
        terms += ",";
        terms.into()
    }
}
