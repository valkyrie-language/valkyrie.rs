use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodDefinition {
    symbol: Vec<Arc<str>>,
    typing: Option<ExpressionKind>,
    span: SourceSpan,
}

impl MethodDefinition {
    pub fn new(name: &IdentifierNode) -> Self {
        todo!()
    }
    pub fn name(&self) -> String {
        todo!()
    }
    pub fn set_type(&mut self, typing: ExpressionKind) {
        self.typing = Some(typing);
    }
    pub fn get_type(&self) -> Option<&ExpressionKind> {
        self.typing.as_ref()
    }
    pub fn get_span(&self) -> SourceSpan {
        self.span
    }
}
