use super::*;

impl ResolveState {
    pub fn resolve(&self) -> Result<CanonicalWasi> {
        let mut output = DependentGraph::default();
        for item in self.items.values() {
            item.to_lir(&mut output, self)?
        }
        Ok(CanonicalWasi::new(output)?)
    }
}

impl Mir2Lir for ModuleItem {
    type Output = ();
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> Result<Self::Output> {
        match self {
            ModuleItem::Structure(s) => s.to_lir(graph, context),
            ModuleItem::Variant(_) => Ok(()),
            ModuleItem::Function(s) => s.to_lir(graph, context),
        }
    }
}
