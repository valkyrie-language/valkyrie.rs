
use super::*;

mod convert;
mod display;

#[derive(Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
pub struct WasmIdentifier {
    /// The namespace of the identifier, only valid when name is not empty
    pub namespace: Vec<Identifier>,
    /// The name of the identifier, anonymous identifier is empty
    pub name: Identifier,
}

impl WasmIdentifier {
    /// Check if it is an anonymous identifier
    pub fn is_anonymous(&self) -> bool {
        self.name.is_empty()
    }
    pub(crate) fn wasi_name(&self) -> String {
        self.name.as_ref().to_case(Case::Kebab)
    }
    pub(crate) fn wasi_id(&self) -> String {
        encode_id(&format!("{self:#}"))
    }
}

impl WasmIdentifier {
    /// Create a new identifier without namespace
    pub fn new(name: Identifier) -> Self
    {
        Self { namespace: Vec::new(), name }
    }
    /// Create a new identifier with current [WasmIdentifier] as namespace
    pub fn join(&self, name: Identifier) -> Self
    {
        match self.name.as_ref() {
            "" => Self { namespace: self.namespace.clone(), name },
            _ => {
                let mut namespace = self.namespace.clone();
                namespace.push(self.name.clone());
                Self { namespace, name }
            }
        }
    }
}
