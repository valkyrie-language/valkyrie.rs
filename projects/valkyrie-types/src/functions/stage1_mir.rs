use super::*;
use valkyrie_ast::ParameterTerm;

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
                println!("FunctionDeclaration: {:?}", self.name);
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

        Ok(FunctionParameter { name, r#type: ValkyrieType::Boolean })
    }
}
