use super::*;

impl Debug for ResolveState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolveContext")
            .field("namespace", &self.namespace.join("∷"))
            .field("document", &self.document)
            .field("items", &self.items.values())
            .field("errors", &self.errors)
            .field("main_function", &self.main_function)
            .finish()
    }
}
impl Debug for ModuleItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function(v) => Debug::fmt(v, f),
            // Self::Imported(v) => Debug::fmt(v, f),
            Self::Structure(v) => Debug::fmt(v, f),
            Self::Variant(v) => Debug::fmt(v, f),
            // Self::Function(v) => Debug::fmt(v, f),
        }
    }
}
