use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::{DependentGraph, WasiFunction, WasiFunctionBody};
impl Mir2Lir for ValkyrieResource {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        // for method in self.methods.values() {
        //     method.to_lir(graph, &self.resource_name)?
        // }
        *graph += WasiResource {
            symbol: self.resource_name.clone(),
            wasi_module: self.wasi_import.module.clone(),
            wasi_name: self.wasi_import.name.clone(),
        };
        Ok(())
    }
}
impl Mir2Lir for ValkyrieClass {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        for method in self.methods.values() {
            method.to_lir(graph, context)?
        }
        // *graph += WasiResource {
        //     symbol: self.class_name.clone(),
        //     wasi_module: import.module.clone(),
        //     wasi_name: import.name.clone(),
        // };
        Ok(())
    }
}
impl Mir2Lir for ValkyrieMethod {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        Ok(())
    }
}
