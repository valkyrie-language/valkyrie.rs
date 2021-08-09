use super::*;

impl crate::DefineImportNode {
    #[allow(unused_imports)]
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ImportStatement> {
        use nyar_error::{ReportKind, SyntaxError};
        let imported = ImportStatement {
            annotation: Default::default(),
            kind: ImportKind::Shared,
            term: self.term(ctx),
            span: ctx.file.with_range(self.span.clone()),
        };

        // for resolved in imported.flatten() {
        //     ctx.add_error(
        //         SyntaxError::new(resolved.to_string())
        //             .with_hint("debug import item")
        //             .with_span(resolved.span)
        //             .as_error(ReportKind::Alert),
        //     )
        // }

        Ok(imported)
    }
    fn term(&self, ctx: &mut ProgramState) -> ImportTermNode {
        if let Some(s) = self.import_term.as_ref().and_then(|v| v.build(ctx)) {
            return s;
        }
        if let Some(s) = &self.import_block {
            return ImportGroupNode::from_iter(s.build(ctx)).into();
        }
        ImportTermNode::default()
    }
}

impl crate::ImportBlockNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Vec<ImportTermNode> {
        let mut terms = Vec::with_capacity(self.import_term.len());
        for term in self.import_term.iter() {
            if let Some(s) = term.build(ctx) {
                terms.push(s)
            }
        }
        terms
    }
}

impl crate::ImportTermNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Option<ImportTermNode> {
        match self {
            Self::ImportAll(v) => Some(v.build(ctx).into()),
            Self::ImportSpace(v) => Some(v.build(ctx).into()),
            Self::ImportName(v) => Some(v.build(ctx).into()),
            Self::EosFree(_) => None,
        }
    }
}

impl crate::ImportAllNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ImportAllNode {
        ImportAllNode { path: self.path.iter().map(|v| v.build(ctx.file)).collect(), span: self.span.clone() }
    }
}

impl crate::ImportSpaceNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ImportGroupNode {
        ImportGroupNode { path: self.path.iter().map(|v| v.build(ctx.file)).collect(), terms: self.body.build(ctx) }
    }
}

impl crate::ImportNameNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ImportAliasNode {
        ImportAliasNode {
            path: self.path.iter().map(|v| v.build(ctx.file)).collect(),
            item: self.item.build(ctx),
            alias: self.alias.build(ctx),
            span: self.span.clone(),
        }
    }
}

impl crate::ImportAsNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Option<ImportAliasItem> {
        Some(self.alias.as_ref()?.build(ctx))
    }
}

impl crate::ImportNameItemNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ImportAliasItem {
        match self {
            Self::ProceduralName(v) => ImportAliasItem::Procedural(v.identifier.build(ctx.file)),
            Self::AttributeName(v) => ImportAliasItem::Attribute(v.identifier.build(ctx.file)),
            Self::Identifier(v) => ImportAliasItem::Normal(v.build(ctx.file)),
        }
    }
}
