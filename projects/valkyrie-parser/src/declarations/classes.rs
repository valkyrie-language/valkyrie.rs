use super::*;

impl crate::DefineClassNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ClassDeclaration> {
        let _ = build_constraint(&self.define_constraint, ctx);
        Ok(ClassDeclaration {
            kind: self.kw_class.build(),
            annotations: self.annotation_head.annotations(ctx),
            name: self.identifier.build(ctx.file),
            generic: None,
            inherits: None,
            implements: vec![],
            terms: self.class_block.build(ctx),
            span: self.span.clone(),
        })
    }
}

impl crate::ClassBlockNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Vec<ClassTerm> {
        let mut terms = Vec::with_capacity(self.class_term.len());
        for term in &self.class_term {
            match term.build(ctx) {
                Ok(s) => terms.extend(s),
                Err(e) => ctx.add_error(e),
            }
        }
        terms
    }
    pub(crate) fn build_domain(&self, ctx: &mut ProgramState) -> DomainDeclaration {
        DomainDeclaration { annotations: Default::default(), body: self.build(ctx), span: self.span.clone() }
    }
}

impl crate::ClassTermNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<Option<ClassTerm>> {
        match self {
            Self::ProceduralCall(v) => Ok(Some(ClassTerm::Macro(v.build(ctx)))),
            Self::DefineDomain(v) => Ok(Some(ClassTerm::Domain(v.build(ctx)?))),
            Self::DefineField(v) => Ok(Some(ClassTerm::Field(v.build(ctx)?))),
            Self::DefineMethod(v) => Ok(Some(ClassTerm::Method(v.build(ctx)?))),
            Self::EosFree(_) => Ok(None),
        }
    }
}

impl crate::KwClassNode {
    pub(crate) fn build(&self) -> ClassKind {
        match self.text.as_str() {
            "class" => ClassKind::Class,
            "structure" => ClassKind::Structure,
            _ => unreachable!(),
        }
    }
}
impl crate::DefineFieldNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<FieldDeclaration> {
        let name = self.identifier.build(ctx.file);
        let annotations = self.annotation_mix.annotations(ctx)?;
        Ok(FieldDeclaration {
            annotations,
            name,
            typing: self.type_hint.build(ctx),
            default: self.parameter_default.build(ctx),
            span: self.span.clone(),
        })
    }
}

impl crate::DefineMethodNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MethodDeclaration> {
        let returns = self.function_middle.returns(ctx)?;
        let annotations = self.annotation_mix.annotations(ctx)?;
        Ok(MethodDeclaration {
            annotations,
            name: self.identifier.build(ctx.file),
            generics: self.function_middle.generics(ctx),
            parameters: self.function_middle.parameters(ctx),
            returns,
            body: self.continuation.as_ref().map(|s| s.build(ctx)),
            span: self.span.clone(),
        })
    }
}

impl crate::DefineDomainNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<DomainDeclaration> {
        Ok(DomainDeclaration { annotations: Default::default(), body: Default::default(), span: self.span.clone() })
    }
}
