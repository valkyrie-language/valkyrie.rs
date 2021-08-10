use super::*;
use valkyrie_ast::ParameterTerm;

impl Hir2Mir for FunctionDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let mut function = ValkyrieFunction {
            function_name: store.register_item(&self.name),
            wasi_import: store.wasi_import_module_name(&self.annotations, &self.name),
            wasi_export: None,
            signature: Default::default(),
        };
        for parameter in self.parameters.positional {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    function.signature.positional.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.mixed {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    function.signature.mixed.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.named {
            match parameter.to_mir(store, &()) {
                Ok(o) => {
                    function.signature.named.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }

        *store += function;

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
