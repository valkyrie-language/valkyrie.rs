use crate::ResolveContext;
use std::sync::Arc;
use valkyrie_ast::NamePathNode;
use valkyrie_lir::{DependentGraph, WasmIdentifier};

pub(crate) trait Hir2Mir {
    type Output;
    type Context<'a>;
    fn to_mir<'a>(self, store: &mut ResolveContext, context: Self::Context<'a>) -> valkyrie_error::Result<Self::Output>;
}

pub(crate) trait Mir2Lir {
    type Output;
    type Context<'a>;
    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_error::Result<Self::Output>;
}

pub(crate) trait AsIdentifier {
    fn as_identifier(&self) -> WasmIdentifier;
}

impl AsIdentifier for NamePathNode {
    fn as_identifier(&self) -> WasmIdentifier {
        match self.path.as_slice() {
            [path @ .., last] => WasmIdentifier {
                namespace: path.iter().map(|x| Arc::from(x.name.as_ref())).collect(),
                name: Arc::from(last.name.as_ref()),
            },
            _ => unreachable!("empty name path"),
        }
    }
}
