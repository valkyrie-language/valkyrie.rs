use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::{DependentGraph, WasiFunction, WasiFunctionBody};

impl Mir2Lir for ValkyrieClass {
    type Output = ();
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> Result<Self::Output> {
        for method in self.methods.values() {
            method.to_lir(graph, self)?
        }
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
impl Mir2Lir for ValkyrieMethod {
    type Output = ();
    type Context = ValkyrieClass;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> Result<Self::Output> {
        let symbol = context.symbol.join(self.method_name.clone());
        match &self.wasi_import {
            Some(s) => {
                *graph += WasiFunction {
                    symbol,
                    inputs: vec![],
                    output: vec![],
                    body: WasiFunctionBody::External { wasi_module: s.module.clone(), wasi_name: s.name.clone() },
                }
            }
            None => {
                *graph += WasiFunction {
                    symbol,
                    inputs: vec![],
                    output: vec![],
                    body: WasiFunctionBody::Normal { bytecodes: vec![] },
                }
            }
        }
        Ok(())
    }
}
