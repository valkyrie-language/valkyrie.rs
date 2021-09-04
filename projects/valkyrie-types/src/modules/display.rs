use super::*;

impl Debug for ResolveContext {
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
impl Debug for NamespaceItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External(v) => Debug::fmt(v, f),
            Self::Function(v) => Debug::fmt(v, f),
            Self::Resource(v) => Debug::fmt(v, f),
            Self::Primitive(v) => Debug::fmt(v, f),
            Self::Structure(v) => Debug::fmt(v, f),
            Self::Flags(v) => Debug::fmt(v, f),
            Self::Enums(v) => Debug::fmt(v, f),
            Self::Variant(v) => Debug::fmt(v, f),
            NamespaceItem::Unknown(_) => {
                unreachable!()
            }
        }
    }
}
