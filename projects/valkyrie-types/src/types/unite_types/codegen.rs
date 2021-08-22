use super::*;
use nyar_wasm::WasiVariantType;

impl AddAssign<ValkyrieUnite> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieUnite) {
        self.items.insert(rhs.unite_name.clone(), ModuleItem::Variant(rhs));
    }
}

impl Mir2Lir for ValkyrieUnite {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        *graph += WasiVariantType { symbol: self.unite_name.clone(), wasi_name: "".to_string(), variants: Default::default() };

        Ok(())
    }
}
