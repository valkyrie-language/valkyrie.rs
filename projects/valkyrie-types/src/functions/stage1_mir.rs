use super::*;

impl Hir2Mir for FunctionDeclaration {
    type Output = ();
    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output> {
        let name = ctx.register_item(&self.name);
        let wasi_import = ctx.wasi_import_module_name(&self.annotations, &self.name);
        *ctx += ValkyrieFunction { function_name: name, wasi_import, wasi_export: None, signature: Default::default() };
        return Ok(());
    }
}
