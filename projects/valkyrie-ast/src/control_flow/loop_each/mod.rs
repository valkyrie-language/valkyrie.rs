use super::*;

mod display;

#[doc = include_str!("readme.md")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopEach {
    /// Position of `each`
    pub keyword: Range<u32>,
    /// `for pattern`
    pub pattern: CasePattern,
    /// `in iterator`
    pub iterator: ExpressionKind,
    /// `if condition`
    pub condition: Option<ExpressionKind>,
    /// `#label`
    pub label: Option<IdentifierNode>,
    /// `{ body }`
    pub body: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

impl ValkyrieNode for LoopEach {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

/// `for ref a, mut b in {...}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ForBarePattern {
    /// The bare tuple pattern
    pub pattern: Vec<ArgumentKey>,
    /// The range of the node
    pub span: Range<u32>,
}

impl LoopEach {
    pub fn standardization(self, iterator: IdentifierNode) -> (VariableDeclaration, LoopRepeat) {
        let var = VariableDeclaration { identifier: iterator, type_hint: None, body: None };
        let lops = LoopRepeat { keyword: self.keyword, label: self.label, terms: vec![] };
        (var, lops)
    }
}

impl ForBarePattern {
    /// Convert this bare pattern into tuple pattern
    #[allow(clippy::wrong_self_convention)]
    pub fn as_pattern_expression(self) -> CasePattern {
        TuplePatternNode {
            bind: None,
            name: None,
            terms: self.pattern.into_iter().map(CasePattern::from).collect(),
            span: self.span,
        }
        .into()
    }
}
