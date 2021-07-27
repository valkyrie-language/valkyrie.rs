use super::*;
use crate::modifiers::AccessType;

mod codegen;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieField {
    pub(crate) symbol: Arc<str>,
    pub(crate) typing: Option<ExpressionKind>,
    readonly: bool,
    pub(crate) optional: bool,
    access: AccessType,
    pub(crate) span: FileSpan,
}

impl ValkyrieField {
    pub fn new(name: &IdentifierNode) -> Self {
        Self {
            symbol: Arc::from(name.name.as_str()),
            typing: None,
            readonly: false,
            optional: false,
            access: AccessType::Public,
            span: Default::default(),
        }
    }
    pub fn get_readonly(&self) -> bool {
        self.readonly
    }
    pub fn set_readonly(&mut self, readonly: bool) {
        self.readonly = readonly;
    }
    pub fn name(&self) -> String {
        self.symbol.to_string()
    }
    pub fn set_type(&mut self, typing: ExpressionKind) {
        self.typing = Some(typing);
    }
    pub fn get_type(&self) -> Option<&ExpressionKind> {
        self.typing.as_ref()
    }
    pub fn get_optional(&self) -> bool {
        self.optional
    }
    pub fn set_optional(&mut self, optional: bool) {
        self.optional = optional;
    }
    pub fn get_span(&self) -> FileSpan {
        self.span
    }
    pub fn set_span(&mut self, span: FileSpan) {
        self.span = span;
    }
}
