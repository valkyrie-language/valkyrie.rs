use super::*;

// static PREFIX: &'static str = r#"^(?x)(
//       [+\-±]
//     | [¬!~]
//     | [⅟√∛∜]
//     | [*]{1,3}
//     | [⁑⁂]
// )"#;

impl<'i> crate::AnnotationMixNode<'i> {
    pub(crate) fn annotations(&self, ctx: &mut ProgramState) -> Result<AnnotationNode> {
        let attributes = build_annotation_terms_mix(&self.annotation_term_mix(), ctx)?;
        let modifiers = ModifierList { terms: self.modifier_ahead().iter().map(|s| s.identifier().build(ctx.file)).collect() };
        Ok(AnnotationNode { documents: DocumentationList { terms: vec![] }, attributes, modifiers })
    }
}

impl<'i> crate::AnnotationHeadNode<'i> {
    pub(crate) fn annotations(&self, ctx: &mut ProgramState) -> AnnotationNode {
        let modifiers = ModifierList { terms: self.modifier_call().iter().map(|s| s.identifier().build(ctx.file)).collect() };
        AnnotationNode {
            documents: DocumentationList { terms: vec![] },
            attributes: build_annotation_terms(&self.annotation_term(), ctx),
            modifiers,
        }
    }
}

impl<'i> crate::AttributeItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> AttributeTerm {
        AttributeTerm {
            kind: Default::default(),
            path: self.namepath().build(ctx),
            variant: vec![],
            arguments: self.arguments(ctx),
            domain: self.domain(ctx),
            span: ctx.file.with_range(self.get_range32()),
        }
    }
    fn domain(&self, ctx: &mut ProgramState) -> Option<StatementBlock> {
        Some(self.continuation().as_ref()?.build(ctx))
    }
    fn arguments(&self, ctx: &mut ProgramState) -> ArgumentsList {
        match &self.tuple_literal() {
            Some(s) => s.to_hir(ctx),
            None => ArgumentsList::default(),
        }
    }
}

impl<'i> crate::AnnotationTermNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> AttributeList {
        let terms = match self {
            Self::AttributeBelowCall(v) => v.attribute_item().iter().map(|v| v.build(ctx)).collect(),
        };
        AttributeList { terms }
    }
}

impl<'i> crate::AnnotationTermMixNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> AttributeList {
        let terms = match self {
            Self::ProceduralCall(v) => vec![v.build(ctx).into()],
            Self::AttributeBelowCall(v) => v.attribute_item().iter().map(|v| v.build(ctx)).collect(),
        };
        AttributeList { terms }
    }
}

impl<'i> crate::ModifierAheadNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> IdentifierNode {
        self.identifier().build(ctx.file)
    }
}
