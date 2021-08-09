use super::*;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieSymbol {
    pub(crate) path: Vec<Arc<str>>,
    pub(crate) span: SourceSpan,
}

impl Display for ValkyrieSymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path.join("∷"))
    }
}

pub(crate) trait AsSymbol {
    fn as_symbol(&self) -> ValkyrieSymbol;
    fn as_namespace_symbol(&self, space: &Option<ValkyrieSymbol>) -> ValkyrieSymbol;
}

impl AsSymbol for NamePathNode {
    fn as_symbol(&self) -> ValkyrieSymbol {
        let path = self.path.iter().map(|s| Arc::from(s.name.as_str())).collect();
        ValkyrieSymbol { path, span: self.span.clone() }
    }

    fn as_namespace_symbol(&self, space: &Option<ValkyrieSymbol>) -> ValkyrieSymbol {
        match space.as_ref() {
            None => self.as_symbol(),
            Some(s) => {
                let mut path = s.path.clone();
                path.extend(self.path.iter().map(|s| Arc::from(s.name.as_str())));
                ValkyrieSymbol { path, span: self.span.clone() }
            }
        }
    }
}
impl AsSymbol for valkyrie_ast::IdentifierNode {
    fn as_symbol(&self) -> ValkyrieSymbol {
        let path = vec![Arc::from(self.name.as_str())];
        ValkyrieSymbol { path, span: self.span.clone() }
    }

    fn as_namespace_symbol(&self, space: &Option<ValkyrieSymbol>) -> ValkyrieSymbol {
        match space.as_ref() {
            None => self.as_symbol(),
            Some(s) => {
                let mut path = s.path.clone();
                path.extend_one(Arc::from(self.name.as_str()));
                ValkyrieSymbol { path, span: self.span.clone() }
            }
        }
    }
}
impl ValkyrieSymbol {
    pub fn new<S: AsSymbol>(id: S) -> Self {
        id.as_symbol()
    }
    pub fn with_span(self, span: SourceSpan) -> Self {
        Self { span, ..self }
    }
}
