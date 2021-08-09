use super::*;
use std::sync::Arc;

impl crate::MatchExpressionNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MatchStatement> {
        Ok(MatchStatement {
            kind: self.kw_match.build(),
            bind: self.get_bind(ctx),
            main: self.inline_expression.build(ctx)?,
            patterns: self.match_block.build(ctx),
            span: self.span.clone(),
        })
    }
    fn get_bind(&self, ctx: &mut ProgramState) -> Option<IdentifierNode> {
        Some(self.identifier.as_ref()?.build(ctx.file))
    }
}

impl crate::MatchBlockNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> PatternsList {
        let mut list = PatternsList::new(self.match_terms.len(), &self.span);
        for term in &self.match_terms {
            match term.build(ctx) {
                Ok(o) => list.branches.extend(o),
                Err(e) => ctx.add_error(e),
            }
        }
        list
    }
}

impl crate::KwMatchNode {
    pub(crate) fn build(&self) -> MatchKind {
        match self {
            Self::Match => MatchKind::Typing,
            Self::Catch => MatchKind::Effect,
        }
    }
}

impl crate::MatchTermsNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<Option<PatternBranch>> {
        let value = match self {
            Self::MatchCase(v) => v.build(ctx)?,
            Self::MatchElse(v) => v.build(ctx)?,
            Self::MatchType(v) => v.build(ctx)?,
            Self::MatchWhen(v) => v.build(ctx)?,
            Self::Comma(_) => return Ok(None),
        };
        Ok(Some(value))
    }
}

impl crate::MatchCaseNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternBranch> {
        Ok(PatternBranch {
            condition: PatternCondition::Case(self.build_node(ctx)?),
            continuation: match_statements(&self.match_statement, ctx),
            span: self.span.clone(),
        })
    }
    fn build_node(&self, ctx: &mut ProgramState) -> Result<PatternCaseNode> {
        Ok(PatternCaseNode { pattern: self.case_pattern.build(ctx)?, guard: self.if_guard.build(ctx), span: self.span.clone() })
    }
}

impl crate::MatchTypeNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternBranch> {
        // avoid block by condition
        let continuation = match_statements(&self.match_statement, ctx);
        Ok(PatternBranch { condition: PatternCondition::Type(self.build_node(ctx)?), continuation, span: self.span.clone() })
    }
    fn build_node(&self, ctx: &mut ProgramState) -> Result<PatternTypeNode> {
        Ok(PatternTypeNode {
            typing: self.type_expression.build(ctx)?,
            guard: self.if_guard.build(ctx),
            span: self.span.clone(),
        })
    }
}

impl crate::MatchWhenNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternBranch> {
        // avoid block by condition
        let continuation = match_statements(&self.match_statement, ctx);
        Ok(PatternBranch { condition: PatternCondition::When(self.build_node(ctx)?), continuation, span: self.span.clone() })
    }
    fn build_node(&self, ctx: &mut ProgramState) -> Result<PatternWhenNode> {
        Ok(PatternWhenNode { guard: self.inline_expression.build(ctx)?, span: self.span.clone() })
    }
}

impl crate::MatchElseNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternBranch> {
        Ok(PatternBranch {
            condition: PatternCondition::Else,
            continuation: match_statements(&self.match_statement, ctx),
            span: self.span.clone(),
        })
    }
}

impl crate::CasePatternNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<PatternNode> {
        match self {
            Self::Namepath(v) => Ok(PatternNode::Atom(Box::new(IdentifierPattern {
                modifiers: Default::default(),
                identifier: IdentifierNode { name: Arc::from(""), span: Default::default() },
            }))),
            Self::StandardPattern(v) => v.build(ctx),
        }
    }
}

fn match_statements(statements: &[crate::MatchStatementNode], ctx: &mut ProgramState) -> StatementBlock {
    let mut list = StatementBlock::new(statements.len(), &Default::default());
    for term in statements {
        match term.statement.build(ctx) {
            Ok(o) => list.terms.extend(o),
            Err(e) => ctx.add_error(e),
        }
    }
    list.update_span();
    list
}
