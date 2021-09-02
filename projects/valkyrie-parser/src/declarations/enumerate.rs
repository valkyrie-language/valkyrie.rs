use super::*;
use yggdrasil_rt::YggdrasilNode;

impl<'i> crate::DefineEnumerateNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<SemanticNumber> {
        let mut terms = vec![];
        for term in &self.flag_term() {
            match term.build(ctx) {
                Ok(s) => terms.extend(s),
                Err(e) => ctx.add_error(e),
            }
        }
        Ok(SemanticNumber {
            annotations: self.annotation_head().annotations(ctx),
            name: self.identifier().build(ctx.file),
            kind: SemanticKind::Enumerate,
            layout: None,
            implements: None,
            body: terms,
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::KwFlagsNode<'i> {
    pub(crate) fn build(&self) -> SemanticKind {
        match self.get_str() {
            "enumerate" | "enum" => SemanticKind::Enumerate,
            "flags" => SemanticKind::Flags,
            _ => unreachable!(),
        }
    }
}
impl<'i> crate::FlagTermNode<'i> {
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

impl<'i> crate::FlagFieldNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<EncodeDeclaration> {
        let mut attributes = AttributeList::new(self.annotation_term().len());
        for x in self.annotation_term().iter() {
            attributes.terms.extend(x.build(ctx).terms)
        }
        let annotations = AnnotationNode { documents: Default::default(), attributes, modifiers: Default::default() };

        Ok(EncodeDeclaration {
            annotations,
            name: self.identifier().build(ctx.file),
            value: self.parameter_default().and_then(|x| x.build(ctx)),
            span: self.get_range32(),
        })
    }
}
