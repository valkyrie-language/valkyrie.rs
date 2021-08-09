use super::*;

impl Mir2Lir for ValkyrieFunction {
    fn to_lir(&self, ctx: &ResolveContext, graph: &mut DependentGraph) -> nyar_error::Result<Self::Output> {
        if let Some(s) = &self.wasi_import {
            *graph += WasiFunction::external(&s.module, &s.name, &self.function_name)
        }
        Ok(())
    }
}
