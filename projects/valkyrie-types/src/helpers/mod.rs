use crate::ResolveState;
use nyar_wasm::{DependentGraph, Identifier};
use valkyrie_ast::NamePathNode;

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

pub(crate) trait AsIdentifier {
    fn as_identifier(&self) -> Identifier;
}

impl AsIdentifier for NamePathNode {
    fn as_identifier(&self) -> Identifier {
        match self.path.as_slice() {
            [path @ .., last] => {
                Identifier { namespace: path.iter().map(|x| x.name.clone()).collect(), name: last.name.clone() }
            }
            _ => unreachable!("empty name path"),
        }
    }
}
