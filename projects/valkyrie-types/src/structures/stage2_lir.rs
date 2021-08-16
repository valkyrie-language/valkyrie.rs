use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::{DependentGraph, WasiFunction, WasiFunctionBody};
impl Mir2Lir for ValkyrieResource {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        for method in self.methods.values() {
            method.to_lir(graph, &self.resource_name)?
        }
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
            method.to_lir(graph, &self.class_name)?
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
    type Context<'a> = &'a Identifier;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        let symbol = context.join(self.method_name.clone());
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
