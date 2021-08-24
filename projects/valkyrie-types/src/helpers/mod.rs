use crate::ResolveState;
use nyar_wasm::DependentGraph;

pub(crate) trait Hir2Mir {
    type Output;
    type Context<'a>;
    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> nyar_error::Result<Self::Output>;
}

pub(crate) trait Mir2Lir {
    type Output;
    type Context<'a>;
    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> nyar_error::Result<Self::Output>;
}
