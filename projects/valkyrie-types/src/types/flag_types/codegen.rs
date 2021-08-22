use super::*;

impl AddAssign<ValkyrieFlagation> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieFlagation) {
        self.items.insert(rhs.flags_name.clone(), ModuleItem::Flags(rhs));
    }
}

impl Mir2Lir for ValkyrieFlagation {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut variants = IndexMap::with_capacity(self.flags.len());
        for (key, value) in self.flags.iter() {
            variants.insert(key.clone(), value.to_lir(graph, context)?);
        }

        *graph += WasiFlags { symbol: self.flags_name.clone(), variants };
        Ok(())
    }
}
