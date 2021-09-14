use super::*;
use nyar_wasm::WasiVariantType;

impl AddAssign<ValkyrieUnite> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieUnite) {
        self.items.insert(rhs.unite_name.clone(), NamespaceItem::Variant(rhs));
    }
}

impl Mir2Lir for ValkyrieUnite {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut variants = Vec::with_capacity(self.variants.len());
        for x in self.variants.values() {
            variants.push(x.to_lir(graph, context)?)
        }
        *graph += WasiVariantType { symbol: self.unite_name.clone(), variants };
        Ok(())
    }
}
