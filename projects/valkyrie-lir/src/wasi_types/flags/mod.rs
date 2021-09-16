use super::*;

mod arithmetic;
mod display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiFlags {
    pub symbol: WasmIdentifier,
    pub flags: Vec<WasiSemanticIndex>,
}
