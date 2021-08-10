use super::*;
use nyar_wasm::{WasiParameter, WasiType};

impl Mir2Lir for ValkyrieFunction {
    type Output = ();
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        if let Some(s) = &self.wasi_import {
            let mut function = WasiFunction::external(&s.module, &s.name, &self.function_name);

            for param in self.signature.positional.values() {
                function.inputs.push(param.to_lir(graph, context)?)
            }

            *graph += function
        }
        Ok(())
    }
}

impl Mir2Lir for FunctionParameter {
    type Output = WasiParameter;
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        Ok(WasiParameter { name: self.name.clone(), wasi_name: self.name.clone(), r#type: WasiType::Boolean })
    }
}
