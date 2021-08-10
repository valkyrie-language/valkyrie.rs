use crate::ResolveState;
use nyar_wasm::DependentGraph;

pub(crate) trait Hir2Mir {
    type Output;
    type Context;
    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output>;
}

pub(crate) trait Mir2Lir {
    type Output;
    type Context;
    fn to_lir(&self, graph: &mut DependentGraph, context: &Self::Context) -> nyar_error::Result<Self::Output>;
}
