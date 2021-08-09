use super::*;

/// `switch { when a > 0: a, else: 0}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SwitchStatement {
    /// The patterns of the switch
    pub patterns: PatternsList,
    /// The range of the node
    pub span: Range<u32>,
}
impl ValkyrieNode for SwitchStatement {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
#[cfg(feature = "pretty-print")]
impl PrettyPrint for SwitchStatement {
    fn pretty(&self, theme: &PrettyProvider) -> PrettyTree {
        let mut terms = PrettySequence::new(10);
        terms += theme.keyword("switch");
        terms += " ";
        terms += self.patterns.pretty(theme);
        terms.into()
    }
}
