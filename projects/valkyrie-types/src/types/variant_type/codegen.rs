use super::*;
use crate::{helpers::Mir2Lir, ValkyrieUnite};
use nyar_wasm::{DependentGraph, WasiVariantItem, WasiVariantType};

impl Mir2Lir for ValkyrieVariant {
    type Output = WasiVariantItem;
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        // *graph +=

        Ok(WasiVariantItem { symbol: self.variant_name.clone(), wasi_name: self.wasi_alias.clone(), fields: None })
    }
}
