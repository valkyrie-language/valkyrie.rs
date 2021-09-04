use super::*;

impl ResolveContext {
    pub fn resolve(&self) -> Result<CanonicalWasi> {
        let mut output = DependentGraph::default();
        for item in self.items.values() {
            item.to_lir(&mut output, self)?
        }
        Ok(CanonicalWasi::new(output)?)
    }
}

impl Mir2Lir for NamespaceItem {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        match self {
            Self::Resource(s) => s.to_lir(graph, context),
            Self::Structure(s) => s.to_lir(graph, context),
            Self::Primitive(s) => s.to_lir(graph, context),
            Self::Variant(s) => s.to_lir(graph, context),
            Self::Function(s) => s.to_lir(graph, context),
            Self::External(s) => s.to_lir(graph, context),
            Self::Flags(s) => s.to_lir(graph, context),
            Self::Enums(s) => s.to_lir(graph, context),
            Self::Unknown(_) => {
                unreachable!()
            }
        }
    }
}
