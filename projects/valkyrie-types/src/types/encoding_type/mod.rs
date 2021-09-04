use super::*;

#[derive(Debug)]
pub struct ValkyrieSemanticNumber {
    pub number_name: Arc<str>,
    pub wasm_alias: Arc<str>,
}
impl Mir2Lir for ValkyrieSemanticNumber {
    type Output = WasiSemanticIndex;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, _: &mut DependentGraph, _: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        Ok(WasiSemanticIndex { name: self.number_name.clone(), wasi_name: self.wasm_alias.clone() })
    }
}
