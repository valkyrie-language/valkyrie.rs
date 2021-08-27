use super::*;

impl<'i> crate::DefineNamespaceNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamespaceDeclaration {
        let kind = match &self.op_namespace() {
            Some(s) => s.build(),
            None => NamespaceKind::Standalone,
        };
        NamespaceDeclaration { kind, path: self.namepath_free().build(ctx), span: self.get_range32() }
    }
}

impl<'i> crate::OpNamespaceNode<'i> {
    pub(crate) fn build(&self) -> NamespaceKind {
        match self {
            Self::Hide(_) => NamespaceKind::Standalone,
            Self::Main(_) => NamespaceKind::Main,
            Self::Test(_) => NamespaceKind::Test,
        }
    }
}
