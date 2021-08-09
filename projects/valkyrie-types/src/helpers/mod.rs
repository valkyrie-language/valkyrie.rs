use crate::ResolveContext;
use nyar_wasm::DependentGraph;

pub(crate) trait Hir2Mir {
    type Output;
    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output>;
}

pub(crate) trait Mir2Lir {
    type Output = ();
    fn to_lir(&self, ctx: &ResolveContext, graph: &mut DependentGraph) -> nyar_error::Result<Self::Output>;
}
