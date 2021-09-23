use super::*;

#[derive(Debug)]
pub struct ValkyrieSemanticNumber {
    pub number_name: Identifier,
    pub wasm_alias: Identifier,
}
impl Mir2Lir for ValkyrieSemanticNumber {
    type Output = WasiSemanticIndex;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, _: &mut DependentGraph, _: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        Ok(WasiSemanticIndex { name: self.number_name.clone(), wasi_name: self.wasm_alias.clone() })
    }
}
