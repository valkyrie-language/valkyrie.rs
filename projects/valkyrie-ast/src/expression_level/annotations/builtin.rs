use super::*;

impl AttributeTerm {
    /// Interpreted as an external function call
    pub fn as_asm(&self) -> Result<&str, NyarError> {
        let code = match self.arguments.terms.get(0) {
            Some(s) => match &s.value {
                ExpressionKind::String(s) => s.literal.text.as_str(),
                _ => Err(NyarError::custom("except string in `asm`"))?,
            },
            None => Err(NyarError::custom("missing module name in `asm`"))?,
        };
        Ok(code)
    }
    /// Interpreted as an external function call
    pub fn as_ffi(&self) -> Result<(&str, &str), NyarError> {
        let module = match self.arguments.terms.get(0) {
            Some(s) => match &s.value {
                ExpressionKind::String(s) => s.literal.text.as_str(),
                _ => Err(NyarError::custom("except string in `ffi`"))?,
            },
            None => Err(NyarError::custom("missing module name in `ffi`"))?,
        };
        let name = match self.arguments.terms.get(1) {
            Some(s) => match &s.value {
                ExpressionKind::String(s) => s.literal.text.as_str(),
                _ => Err(NyarError::custom("except string in `ffi`"))?,
            },
            None => Err(NyarError::custom("missing field name in ffi name in `ffi`"))?,
        };
        Ok((module, name))
    }
}
