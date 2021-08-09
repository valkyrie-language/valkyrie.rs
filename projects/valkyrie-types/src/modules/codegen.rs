use super::*;

impl ResolveContext {
    pub fn resolve(&self) -> Result<CanonicalWasi> {
        let mut output = DependentGraph::default();
        for item in self.items.values() {
            item.to_lir(self, &mut output)?
        }
        Ok(CanonicalWasi::new(output)?)
    }
}

impl Mir2Lir for ModuleItem {
    fn to_lir(&self, ctx: &ResolveContext, graph: &mut DependentGraph) -> Result<Self::Output> {
        match self {
            ModuleItem::Structure(s) => s.to_lir(ctx, graph),
            ModuleItem::Variant(_) => Ok(()),
            ModuleItem::Function(s) => s.to_lir(ctx, graph),
        }
    }
}
