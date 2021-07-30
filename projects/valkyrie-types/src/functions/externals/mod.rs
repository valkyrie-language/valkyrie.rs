use super::*;
use nyar_wasm::WasmParameter;
use std::ops::AddAssign;
use valkyrie_ast::{ExpressionKind, ParameterTerm};

mod codegen;

#[derive(Clone, Debug)]
pub struct ValkyrieExternalFunction {
    name: ValkyrieSymbol,
    bind_module: Arc<str>,
    bind_name: Arc<str>,
    inputs: Vec<ParameterTerm>,
    outputs: Vec<ExpressionKind>,
}

impl AddAssign<ParameterTerm> for ValkyrieExternalFunction {
    fn add_assign(&mut self, rhs: ParameterTerm) {
        self.inputs.push(rhs)
    }
}

impl ValkyrieExternalFunction {
    /// Create a new external function
    pub fn new(name: ValkyrieSymbol) -> Self {
        Self { name, bind_module: Arc::from(""), bind_name: Arc::from(""), inputs: vec![], outputs: vec![] }
    }
    pub fn set_path(&mut self, module: &str, name: &str) {
        self.bind_module = Arc::from(module);
        self.bind_name = Arc::from(name);
    }
}
