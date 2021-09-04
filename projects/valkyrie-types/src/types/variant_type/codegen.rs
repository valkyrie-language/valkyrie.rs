use super::*;

impl Mir2Lir for ValkyrieVariant {
    type Output = WasiVariantItem;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        // *graph +=

        Ok(WasiVariantItem { symbol: self.variant_name.clone(), wasi_name: self.wasi_alias.clone(), fields: None })
    }
}
