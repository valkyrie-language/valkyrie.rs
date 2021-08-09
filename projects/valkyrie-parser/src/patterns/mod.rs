use crate::helpers::ProgramState;
use nyar_error::Result;
use std::sync::Arc;
use valkyrie_ast::*;

impl crate::LetPatternNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        match self {
            Self::BarePattern(v) => v.build(ctx),
            Self::StandardPattern(v) => v.build(ctx),
        }
    }
}
impl crate::StandardPatternNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        match self {
            Self::TuplePattern(v) => v.build(ctx),
        }
    }
}

impl crate::BarePatternNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        let mut terms = vec![];
        for node in &self.bare_pattern_item {
            match node.build(ctx) {
                Ok(o) => terms.push(o),
                Err(e) => ctx.add_error(e),
            }
        }
        let tuple = TuplePatternNode { bind: None, name: None, terms, span: Default::default() };
        Ok(PatternNode::Tuple(Box::new(tuple)))
    }
}

impl crate::BarePatternItemNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        let identifier = self.identifier.build(ctx.file);
        let id = IdentifierPattern { modifiers: Default::default(), identifier };
        Ok(PatternNode::Atom(Box::new(id)))
    }
}

impl crate::TuplePatternNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        let mut terms = vec![];
        for node in &self.pattern_item {
            match node.build(ctx) {
                Ok(o) => terms.push(o),
                Err(e) => return Err(e),
            }
        }
        let tuple = TuplePatternNode { bind: None, name: None, terms, span: Default::default() };
        Ok(PatternNode::Tuple(Box::new(tuple)))
    }
}
impl crate::PatternItemNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        let value = match self {
            Self::OmitDict => PatternNode::Atom(Box::new(IdentifierPattern {
                modifiers: Default::default(),
                identifier: IdentifierNode { name: Arc::from(""), span: Default::default() },
            })),
            Self::OmitList => PatternNode::Atom(Box::new(IdentifierPattern {
                modifiers: Default::default(),
                identifier: IdentifierNode { name: Arc::from(""), span: Default::default() },
            })),
            Self::TuplePatternItem(v) => v.build(ctx)?,
        };
        Ok(value)
    }
}

impl crate::TuplePatternItemNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        let identifier = self.identifier.build(ctx.file);
        let id = IdentifierPattern { modifiers: Default::default(), identifier };
        Ok(PatternNode::Atom(Box::new(id)))
    }
}
