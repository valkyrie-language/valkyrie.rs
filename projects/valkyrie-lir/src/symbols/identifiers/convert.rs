use valkyrie_types::SyntaxError;
use super::*;

impl From<&str> for WasmIdentifier {
    fn from(value: &str) -> Self {
        Self::from_str(value).unwrap()
    }
}

impl From<Arc<str>> for WasmIdentifier {
    fn from(value: Arc<str>) -> Self {
        Self::from_str(&value).unwrap()
    }
}
impl FromStr for WasmIdentifier {
    type Err = SyntaxError;

    /// `package::module::name`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let names: Vec<_> = s.split("::").map(Identifier::new).collect();
        match names.as_slice() {
            [] => Err(SyntaxError::new("empty identifier")),
            [name] => Ok(WasmIdentifier { namespace: vec![], name: name.clone() }),
            [path @ .., name] => Ok(WasmIdentifier { namespace: path.to_vec(), name: name.clone() }),
        }
    }
}
