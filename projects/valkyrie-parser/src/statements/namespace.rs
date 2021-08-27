use super::*;

impl crate::DefineNamespaceNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamespaceDeclaration {
        let kind = match &self.op_namespace {
            Some(s) => s.build(),
            None => NamespaceKind::Standalone,
        };
        NamespaceDeclaration { kind, path: self.namepath_free.build(ctx), span: self.get_range32() }
    }
}

impl crate::OpNamespaceNode {
    pub(crate) fn build(&self) -> NamespaceKind {
        match self {
            Self::Hide => NamespaceKind::Standalone,
            Self::Main => NamespaceKind::Main,
            Self::Test => NamespaceKind::Test,
        }
    }
}
