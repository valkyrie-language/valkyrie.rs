use super::*;
use nyar_error::NyarError;

mod codegen;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDefinition {
    symbol: Vec<Arc<str>>,
    parameters: IndexMap<String, ParameterDefinition>,
    span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterDefinition {
    symbol: Arc<str>,
    span: SourceSpan,
}

impl FunctionDefinition {
    pub fn new(namespace: &NamePathNode, name: &NamePathNode) -> Result<Self> {
        if name.path.is_empty() {
            Err(NyarError::custom("function name cannot be empty"))?
        }
        // SAFETY: `name.path` is not empty
        let span = unsafe { name.path.get_unchecked(name.path.len() - 1).span };
        let mut symbol = Vec::with_capacity(namespace.path.len() + name.path.len());
        symbol.extend(namespace.path.iter().map(|s| Arc::from(s.name.as_str())));
        symbol.extend(name.path.iter().map(|s| Arc::from(s.name.as_str())));
        Ok(Self { symbol, parameters: Default::default(), span })
    }
    pub fn name(&self) -> String {
        self.symbol.join("∷")
    }
    pub fn set_parameter(&mut self, typing: ParameterDefinition) {
        self.parameters.insert(typing.name(), typing);
    }
    pub fn get_parameter(&self, name: &str) -> Option<&ParameterDefinition> {
        self.parameters.get(name)
    }
    pub fn get_span(&self) -> SourceSpan {
        self.span
    }
}

impl ParameterDefinition {
    pub fn new(name: &IdentifierNode) -> Self {
        Self { symbol: Arc::from(name.name.as_str()), span: name.span }
    }
    pub fn name(&self) -> String {
        self.symbol.to_string()
    }
}
