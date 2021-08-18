use super::*;
use valkyrie_ast::{ExpressionKind, IdentifierNode, ParameterTerm};
use valkyrie_parser::NamepathNode;

impl Hir2Mir for FunctionDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let function_name = store.register_item(&self.name);
        let mut signature = FunctionSignature::default();

        for parameter in self.parameters.positional {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    signature.positional.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.mixed {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    signature.mixed.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.named {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    signature.named.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        match store.wasi_import_module_name(&self.annotations, &self.name) {
            Some(wasi_import) => {
                *store += ValkyrieImportFunction { function_name, wasi_import, signature };
            }
            None => {
                let mut overloads = BTreeMap::default();
                unsafe {
                    overloads.insert(NotNan::new_unchecked(0.0), signature);
                }
                *store += ValkyrieNativeFunction { function_name, wasi_export: None, overloads };
            }
        }

        return Ok(());
    }
}

impl Hir2Mir for ParameterTerm {
    type Output = FunctionParameter;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let name = self.key.name;
        let type_hint = match self.bound {
            Some(s) => match s {
                ExpressionKind::Symbol(s) => match s.path.as_slice() {
                    [single] => match single.name.as_ref() {
                        "bool" | "Boolean" => ValkyrieType::Boolean,
                        "u8" => ValkyrieType::Unsigned { bits: 8 },
                        "u16" => ValkyrieType::Unsigned { bits: 16 },
                        "u32" => ValkyrieType::Unsigned { bits: 32 },
                        "u64" => ValkyrieType::Unsigned { bits: 64 },
                        "i8" => ValkyrieType::Integer { bits: 8 },
                        "i16" => ValkyrieType::Integer { bits: 16 },
                        "i32" => ValkyrieType::Integer { bits: 32 },
                        "i64" => ValkyrieType::Integer { bits: 64 },
                        "f32" => ValkyrieType::Float { bits: 32 },
                        "f64" => ValkyrieType::Float { bits: 64 },
                        "char" => ValkyrieType::Unicode,
                        _ => Err(nyar_error::SyntaxError::new("Unknown Type hint for parameter")
                            .with_hint(format!("{:?}", s))
                            .with_span(self.key.span))?,
                    },
                    long => Err(nyar_error::SyntaxError::new("Unknown Type hint for parameter")
                        .with_hint(format!("{:?}", s))
                        .with_span(self.key.span))?,
                },
                _ => Err(nyar_error::SyntaxError::new("Invalid type hint for parameter")
                    .with_hint(format!("{:?}", s))
                    .with_span(self.key.span))?,
            },
            None => Err(nyar_error::SyntaxError::new("Missing type hint for parameter").with_span(self.key.span))?,
        };
        Ok(FunctionParameter { name, r#type: type_hint })
    }
}
