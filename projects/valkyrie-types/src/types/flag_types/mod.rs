use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::{DependentGraph, WasiFlags};

#[derive(Debug)]
pub struct ValkyrieFlagation {
    pub flags_name: Identifier,
    pub flags: IndexMap<Arc<str>, ValkyrieSemanticNumber>,
}

impl AddAssign<ValkyrieFlagation> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieFlagation) {
        self.items.insert(rhs.flags_name.clone(), ModuleItem::Flags(rhs));
    }
}

impl Mir2Lir for ValkyrieFlagation {
    type Output = ();
    type Context<'a> = &'a ResolveState;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        println!("Flag: {}", self.flags_name);
        *graph += WasiFlags { symbol: self.flags_name.clone(), variants: Default::default() };
        Ok(())
    }
}
