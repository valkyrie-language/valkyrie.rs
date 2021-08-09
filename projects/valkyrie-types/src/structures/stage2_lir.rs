use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::DependentGraph;

impl Mir2Lir for ValkyrieClass {
    fn to_lir(&self, ctx: &ResolveContext, graph: &mut DependentGraph) -> Result<Self::Output> {
        for method in self.methods.values() {}

        match &self.wasi_import {
            Some(import) => {
                *graph += WasiResource {
                    symbol: self.symbol.clone(),
                    wasi_module: import.module.clone(),
                    wasi_name: import.name.clone(),
                };
            }
            None => {}
        }
        Ok(())
    }
}
