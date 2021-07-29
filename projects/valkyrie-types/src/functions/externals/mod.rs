use super::*;
use nyar_wasm::ParameterType;
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

impl ValkyrieExternalFunction {
    /// Create a new external function
    pub fn new(name: ValkyrieSymbol) -> Self {
        Self { name, bind_module: Arc::from(""), bind_name: Arc::from(""), inputs: vec![], outputs: vec![] }
    }
    pub fn set_path(&mut self, module: &str, name: &str) {
        self.bind_module = Arc::from(module);
        self.bind_name = Arc::from(name);
    }
    pub fn add_parameter(&mut self, e: ParameterTerm) {
        self.inputs.push(e)
    }
}
