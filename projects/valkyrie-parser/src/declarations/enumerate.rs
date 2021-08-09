use super::*;

impl crate::DefineEnumerateNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<FlagDeclaration> {
        let mut terms = vec![];
        for term in &self.flag_term {
            match term.build(ctx) {
                Ok(s) => terms.extend(s),
                Err(e) => ctx.add_error(e),
            }
        }
        Ok(FlagDeclaration {
            annotations: self.annotation_head.annotations(ctx),
            name: self.identifier.build(ctx.file),
            kind: self.kw_flags.build(),
            layout: None,
            implements: None,
            body: terms,
            span: self.span.clone(),
        })
    }
}

impl crate::KwFlagsNode {
    pub(crate) fn build(&self) -> FlagKind {
        match self.text.as_str() {
            "enumerate" | "enum" => FlagKind::Enumerate,
            "flags" => FlagKind::Flags,
            _ => unreachable!(),
        }
    }
}
impl crate::FlagTermNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<Option<FlagTerm>> {
        let value = match self {
            Self::ProceduralCall(v) => v.build(ctx).into(),
            Self::DefineMethod(v) => v.build(ctx)?.into(),
            Self::FlagField(v) => v.build(ctx)?.into(),
            Self::EosFree(_) => return Ok(None),
        };
        Ok(Some(value))
    }
}

impl crate::FlagFieldNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<EncodeDeclaration> {
        Ok(EncodeDeclaration {
            annotations: Default::default(),
            name: self.identifier.build(ctx.file),
            value: self.parameter_default.build(ctx),
            span: self.span.clone(),
        })
    }
}
