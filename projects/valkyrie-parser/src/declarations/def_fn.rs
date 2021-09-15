use super::*;
use valkyrie_error::{ReportKind, SourceSpan, SyntaxError};
use yggdrasil_rt::YggdrasilNode;

impl<'i> crate::DefineFunctionNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<FunctionDeclaration> {
        Ok(FunctionDeclaration {
            keyword: self.kw_function().get_span(ctx),
            kind: self.kw_function().build(ctx),
            annotations: self.annotation_head().annotations(ctx),
            name: self.identifier().build(ctx.file),
            generics: self.function_middle().generics(ctx),
            parameters: self.function_middle().parameters(ctx),
            returns: self.function_middle().returns(ctx)?,
            body: self.continuation().build(ctx),
        })
    }
}

impl<'i> crate::KwFunctionNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> FunctionKind {
        match self.get_str() {
            "macro" => FunctionKind::Macro,
            "micro" => FunctionKind::Micro,
            deprecated @ ("function" | "func" | "fun" | "fn") => {
                ctx.add_error(
                    SyntaxError::new(format!("Using `{deprecated}` to declare micro functions has been deprecated"))
                        .with_hint("use `micro` instead")
                        .with_span(ctx.file.with_range(self.get_range32()))
                        .as_error(ReportKind::Alert),
                );
                FunctionKind::Micro
            }
            _ => unreachable!(),
        }
    }
    pub(crate) fn get_span(&self, ctx: &mut ProgramState) -> SourceSpan {
        ctx.file.with_range(self.get_range32())
    }
}

impl<'i> crate::FunctionMiddleNode<'i> {
    pub(crate) fn returns(&self, ctx: &mut ProgramState) -> Result<FunctionReturnNode> {
        let typing = match &self.type_return() {
            Some(s) => Some(s.type_expression().build(ctx)?),
            None => None,
        };
        let effect = match &self.type_effect() {
            Some(_) => {
                vec![]
            }
            None => {
                vec![]
            }
        };

        Ok(FunctionReturnNode { typing, effect })
    }
    pub(crate) fn parameters(&self, ctx: &mut ProgramState) -> ParametersList {
        self.function_parameters().build(ctx)
    }
    pub(crate) fn generics(&self, ctx: &mut ProgramState) -> ParametersList {
        let mut list = ParametersList::new(ParameterKind::Generic);
        let mut terms = vec![];
        match &self.define_generic() {
            Some(s) => {
                for term in &s.generic_parameter().generic_parameter_pair() {
                    match term.build(ctx) {
                        Ok(s) => terms.push(s),
                        Err(e) => ctx.add_error(e),
                    }
                }
            }
            None => {}
        }
        list.resolve(terms).into_iter().for_each(|e| ctx.add_error(e));
        list
    }
}

impl<'i> crate::FunctionParametersNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ParametersList {
        let mut list = ParametersList::new(ParameterKind::Expression);
        let mut terms = vec![];
        for term in &self.parameter_item() {
            match term.build(ctx) {
                Ok(s) => terms.push(s),
                Err(e) => ctx.add_error(e),
            }
        }
        list.resolve(terms).into_iter().for_each(|e| ctx.add_error(e));
        list
    }
}

impl<'i> crate::GenericParameterNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ParametersList {
        let mut list = ParametersList::new(ParameterKind::Generic);
        let mut terms = vec![];
        for term in &self.generic_parameter_pair() {
            match term.build(ctx) {
                Ok(s) => terms.push(s),
                Err(e) => ctx.add_error(e),
            }
        }
        list.resolve(terms).into_iter().for_each(|e| ctx.add_error(e));
        list
    }
}

impl<'i> crate::DefineGenericNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ParametersList {
        self.generic_parameter().build(ctx)
    }
}

impl<'i> crate::GenericParameterPairNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ParameterMixed> {
        let key = self.identifier().build(ctx.file);
        Ok(ParameterMixed::Term(ParameterTerm {
            annotations: Default::default(),
            unpack: 0,
            key,
            bound: self.build_bound(ctx),
            default: self.build_default(ctx),
        }))
    }
    fn build_bound(&self, ctx: &mut ProgramState) -> Option<ExpressionKind> {
        match self.bound().as_ref()?.build(ctx) {
            Ok(o) => Some(o),
            Err(e) => {
                ctx.add_error(e);
                None
            }
        }
    }
    fn build_default(&self, ctx: &mut ProgramState) -> Option<ExpressionKind> {
        match self.default().as_ref()?.build(ctx) {
            Ok(o) => Some(o),
            Err(e) => {
                ctx.add_error(e);
                None
            }
        }
    }
}
impl<'i> crate::ParameterItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ParameterMixed> {
        let value = match self {
            Self::ParameterPair(v) => v.build(ctx)?,
            Self::ParameterItemControl(v) => match v.get_str() {
                "「" | "<" => ParameterMixed::LMark(ctx.file.with_range(v.get_range32())),
                "」" | ">" => ParameterMixed::RMark(ctx.file.with_range(v.get_range32())),
                ".." => {
                    todo!()
                }
                "..." => {
                    todo!()
                }
                _ => {
                    todo!()
                }
            },
        };
        Ok(value)
    }
}

impl<'i> crate::ParameterPairNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ParameterMixed> {
        let key = self.identifier().build(ctx.file);
        Ok(ParameterMixed::Term(ParameterTerm {
            annotations: self.annotations(ctx),
            unpack: 0,
            key,
            bound: self.type_hint().and_then(|x| x.build(ctx)),
            default: self.parameter_default().and_then(|x| x.build(ctx)),
        }))
    }
    fn annotations(&self, ctx: &mut ProgramState) -> AnnotationNode {
        let mut out = AnnotationNode::default();
        out.modifiers = build_modifier_ahead(&self.modifier_ahead(), ctx);
        out
    }
}
impl<'i> crate::ParameterDefaultNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Option<ExpressionKind> {
        let expr = self.default().build(ctx);
        match expr {
            Ok(o) => Some(o),
            Err(e) => {
                ctx.add_error(e);
                None
            }
        }
    }
}

impl<'i> crate::ContinuationNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> StatementBlock {
        let mut out = StatementBlock::new(self.statement().len(), &self.get_range32());
        for term in &self.statement() {
            match term.build(ctx) {
                Ok(s) => out.terms.extend(s),
                Err(e) => ctx.add_error(e),
            }
        }
        out
    }
}
impl<'i> crate::TypeHintNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Option<ExpressionKind> {
        let hint = self.hint().build(ctx);
        match hint {
            Ok(o) => Some(o),
            Err(e) => {
                ctx.add_error(e);
                None
            }
        }
    }
}
