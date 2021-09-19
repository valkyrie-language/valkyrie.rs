use super::*;
use valkyrie_types::NyarError;

impl MethodDeclaration {
    pub fn as_assembly(&self) -> Result<Option<FunctionAssembly>, NyarError> {
        if !self.annotations.modifiers.contains("asm") {
            return Ok(None);
        }

        match &self.body {
            Some(s) => match s.terms.as_slice() {
                [] => Ok(Some(FunctionAssembly { text: "".to_string() })),
                [node] => match node {
                    StatementKind::Expression(v) => match &v.body {
                        ExpressionKind::Text(x) => {
                            todo!()
                        }
                        ExpressionKind::String(x) => {
                            todo!()
                        }
                        ExpressionKind::Formatted(x) => {
                            todo!()
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                },
                _ => todo!(),
            },
            None => Ok(Some(FunctionAssembly { text: "".to_string() })),
        }
    }
}
