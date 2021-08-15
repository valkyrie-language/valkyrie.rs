use super::*;
use nyar_wasm::{WasiParameter, WasiType};

impl Mir2Lir for ValkyrieImportFunction {
    type Output = ();
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let mut function = WasiFunction::external(&self.wasi_import.module, &self.wasi_import.name, &self.function_name);
        for param in self.signature.positional.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }
        *graph += function;
        Ok(())
    }
}

impl Mir2Lir for ValkyrieNativeFunction {
    type Output = ();
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        unimplemented!()
    }
}

impl Mir2Lir for FunctionParameter {
    type Output = WasiParameter;
    type Context = ResolveState;

    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        Ok(WasiParameter { name: self.name.clone(), wasi_name: self.name.clone(), r#type: WasiType::Boolean })
    }
}
