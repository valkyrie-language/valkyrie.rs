use valkyrie_types::Identifier;
use super::*;

mod arithmetic;
mod display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiEnumeration {
    pub symbol: WasmIdentifier,
    pub enumerations: Vec<WasiSemanticIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WasiSemanticIndex {
    pub name: Identifier,
    pub wasi_name: Identifier,
}
