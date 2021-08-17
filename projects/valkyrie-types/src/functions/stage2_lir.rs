use super::*;
use nyar_wasm::{WasiFunctionBody, WasiParameter, WasiType};
use std::mem::transmute;

impl Mir2Lir for ValkyrieImportFunction {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut function = WasiFunction::external(&self.wasi_import.module, &self.wasi_import.name, &self.function_name);
        for param in self.signature.positional.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }
        for param in self.signature.mixed.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }
        for param in self.signature.named.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }

        *graph += function;
        Ok(())
    }
}

impl Mir2Lir for ValkyrieNativeFunction {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        for (id, f) in self.overloads.iter() {
            let name = if id.eq(&0.0) {
                self.function_name.clone()
            }
            else {
                self.function_name.join(format!("0x{:X}", unsafe { transmute::<f64, u64>(id.into_inner()) }))
            };
            let inputs = f.to_lir(graph, context)?;
            *graph +=
                WasiFunction { symbol: name, inputs, output: vec![], body: WasiFunctionBody::Normal { bytecodes: vec![] } };
        }

        Ok(())
    }
}

impl Mir2Lir for FunctionSignature {
    type Output = Vec<WasiParameter>;
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let mut outs = Vec::with_capacity(16);
        for param in self.positional.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        for param in self.mixed.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        for param in self.named.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        Ok(outs)
    }
}

impl Mir2Lir for FunctionParameter {
    type Output = WasiParameter;
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        Ok(WasiParameter { name: self.name.clone(), wasi_name: self.name.clone(), r#type: WasiType::Boolean })
    }
}
