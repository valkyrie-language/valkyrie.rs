use super::*;

impl AddAssign<ValkyrieFlagation> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieFlagation) {
        self.items.insert(rhs.flags_name.clone(), NamespaceItem::Flags(rhs));
    }
}

impl Mir2Lir for ValkyrieFlagation {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut flags = Vec::with_capacity(self.flags.len());
        for value in self.flags.values() {
            flags.push(value.to_lir(graph, context)?);
        }
        *graph += WasiFlags { symbol: self.flags_name.clone(), flags };
        Ok(())
    }
}
