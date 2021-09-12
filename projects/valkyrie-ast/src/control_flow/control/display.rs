use super::*;

impl ControlKind {
    /// Convert to keywords
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Goto => "goto",
            Self::Raise => "raise",
            Self::Break => "break",
            Self::Continue => "continue",
            Self::Fallthrough => "fallthrough",
            Self::FallthroughUnchecked => "fallthrough!",
            Self::Return => "return",
            Self::Resume => "resume",
            Self::YieldReturn => "yield return",
            Self::YieldBreak => "yield break",
            Self::YieldFrom => "yield from",
            Self::YieldSend => "yield wait",
            Self::Await => "await",
            Self::AwaitNever => "await?",
            Self::AwaitBlockOn => "await!",
        }
    }
}

impl Display for ControlKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for ControlNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let w = &mut f.debug_struct("Control");
        w.field("kind", &self.kind);
        match &self.label {
            LabelNode::Nearest => {}
            LabelNode::Named(s) => {
                w.field("label", &s.name.to_string());
            }
        }
        if let Some(e) = &self.expression {
            w.field("expression", e);
        }
        w.field("span", &self.span);
        w.finish()
    }
}

#[cfg(feature = "pretty-print")]
impl PrettyPrint for RaiseNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(2);
        terms += theme.keyword("raise");
        terms += " ";
        if let Some(s) = &self.expression {
            terms += s.pretty(theme);
        }
        terms.into()
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for ControlNode {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(3);
        terms += self.r#type.pretty(theme);
        if let Some(s) = &self.expression {
            terms += " ";
            terms += s.pretty(theme);
        }
        terms.into()
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for ControlKind {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        theme.keyword(self.as_str())
    }
}
