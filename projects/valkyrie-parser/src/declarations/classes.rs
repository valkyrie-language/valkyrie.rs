use super::*;

impl<'i> crate::DefineClassNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ClassDeclaration> {
        let _ = build_constraint(&self.define_constraint(), ctx);
        Ok(ClassDeclaration {
            keyword: self.class_kind().get_range32(),
            kind: self.class_kind().build(),
            annotations: self.annotation_head().annotations(ctx),
            name: self.identifier().build(ctx.file),
            generic: None,
            inherits: None,
            implements: vec![],
            terms: self.class_block().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::ClassBlockNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Vec<ClassTerm> {
        let mut terms = Vec::with_capacity(self.class_term().len());
        for term in &self.class_term() {
            match term.build(ctx) {
                Ok(s) => terms.extend(s),
                Err(e) => ctx.add_error(e),
            }
        }
        terms
    }
    pub(crate) fn build_domain(&self, ctx: &mut ProgramState) -> DomainDeclaration {
        DomainDeclaration { annotations: Default::default(), body: self.build(ctx), span: self.get_range32() }
    }
}

impl<'i> crate::ClassTermNode<'i> {
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

impl<'i> crate::ClassKindNode<'i> {
    pub(crate) fn build(&self) -> ClassKind {
        match self {
            Self::KwClass(_) => ClassKind::Class,
            Self::KwStructure(_) => ClassKind::Structure,
            Self::KwWidget(_) => {
                unreachable!()
            }
            Self::KwNeural(_) => {
                unreachable!()
            }
        }
    }
}
impl<'i> crate::DefineFieldNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<FieldDeclaration> {
        let name = self.identifier().build(ctx.file);
        let annotations = self.annotation_mix().annotations(ctx)?;
        Ok(FieldDeclaration {
            annotations,
            name,
            typing: self.type_hint().and_then(|x| x.build(ctx)),
            default: self.parameter_default().and_then(|x| x.build(ctx)),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::DefineMethodNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MethodDeclaration> {
        let returns = self.function_middle().returns(ctx)?;
        let annotations = self.annotation_mix().annotations(ctx)?;
        Ok(MethodDeclaration {
            annotations,
            name: self.identifier().build(ctx.file),
            generics: self.function_middle().generics(ctx),
            parameters: self.function_middle().parameters(ctx),
            returns,
            body: self.continuation().as_ref().map(|s| s.build(ctx)),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::DefineDomainNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<DomainDeclaration> {
        Ok(DomainDeclaration { annotations: Default::default(), body: Default::default(), span: self.get_range32() })
    }
}
