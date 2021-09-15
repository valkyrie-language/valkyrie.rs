use crate::helpers::ProgramState;
use valkyrie_error::Result;
use std::sync::Arc;
use valkyrie_ast::*;

impl<'i> crate::LetPatternNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        match self {
            Self::BarePattern(v) => v.build(ctx),
            Self::StandardPattern(v) => v.build(ctx),
        }
    }
}
impl<'i> crate::StandardPatternNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        match self {
            Self::TuplePattern(v) => v.build(ctx),
        }
    }
}

impl<'i> crate::BarePatternNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        let mut terms = vec![];
        for node in &self.bare_pattern_item() {
            match node.build(ctx) {
                Ok(o) => terms.push(o),
                Err(e) => ctx.add_error(e),
            }
        }
        let tuple = TuplePatternNode { bind: None, name: None, terms, span: Default::default() };
        Ok(CasePattern::Tuple(Box::new(tuple)))
    }
}

impl<'i> crate::BarePatternItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        let identifier = self.identifier().build(ctx.file);
        let id = IdentifierPattern { modifiers: Default::default(), identifier };
        Ok(CasePattern::Atom(Box::new(id)))
    }
}

impl<'i> crate::TuplePatternNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        let mut terms = vec![];
        for node in &self.pattern_item() {
            match node.build(ctx) {
                Ok(o) => terms.push(o),
                Err(e) => return Err(e),
            }
        }
        let tuple = TuplePatternNode { bind: None, name: None, terms, span: Default::default() };
        Ok(CasePattern::Tuple(Box::new(tuple)))
    }
}
impl<'i> crate::PatternItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        let value = match self {
            Self::OmitDict(_) => CasePattern::Atom(Box::new(IdentifierPattern {
                modifiers: Default::default(),
                identifier: IdentifierNode { name: Identifier::new(""), span: Default::default(), shadow_index: 0 },
            })),
            Self::OmitList(_) => CasePattern::Atom(Box::new(IdentifierPattern {
                modifiers: Default::default(),
                identifier: IdentifierNode { name: Identifier::new(""), span: Default::default(), shadow_index: 0 },
            })),
            Self::TuplePatternItem(v) => v.build(ctx)?,
        };
        Ok(value)
    }
}

impl<'i> crate::TuplePatternItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CasePattern> {
        let identifier = self.identifier().build(ctx.file);
        let id = IdentifierPattern { modifiers: Default::default(), identifier };
        Ok(CasePattern::Atom(Box::new(id)))
    }
}
