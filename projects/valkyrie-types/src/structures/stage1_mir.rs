use super::*;

impl Hir2Mir for ClassDeclaration {
    type Output = ();
    fn to_mir(self, ctx: &mut ResolveContext) -> Result<Self::Output> {
        let symbol = ctx.register_item(&self.name);
        let mut class = ValkyrieClass::new(symbol);
        class.wasi_import = ctx.wasi_import_module_name(&self.annotations, &self.name);
        for x in self.terms {
            match x {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(ctx)?;
                    match class.fields.insert(field.field_name.clone(), field) {
                        Some(s) => {
                            unimplemented!()
                        }
                        None => {}
                    }
                }
                ClassTerm::Method(v) => {
                    let method = v.to_mir(ctx)?;
                    match class.methods.insert(method.method_name.clone(), method) {
                        Some(s) => {
                            unimplemented!()
                        }
                        None => {}
                    }
                }
                ClassTerm::Domain(_) => {
                    todo!()
                }
            }
        }
        *ctx += class;

        Ok(())
    }
}

impl AddAssign<ValkyrieClass> for ResolveContext {
    fn add_assign(&mut self, rhs: ValkyrieClass) {
        self.items.insert(rhs.symbol.clone(), ModuleItem::Structure(rhs));
    }
}

impl Hir2Mir for FieldDeclaration {
    type Output = ValkyrieField;
    fn to_mir(self, ctx: &mut ResolveContext) -> Result<Self::Output> {
        let (field_name, wasi_alias) = ctx.export_field(&self.name, &self.annotations)?;

        Ok(ValkyrieField { field_name, wasi_alias })
    }
}
impl Hir2Mir for MethodDeclaration {
    type Output = ValkyrieMethod;
    fn to_mir(self, ctx: &mut ResolveContext) -> Result<Self::Output> {
        let wasi_import = ctx.wasi_import_module_name(&self.annotations, &self.name);
        Ok(ValkyrieMethod { method_name: self.name.name.clone(), wasi_import, wasi_export: None })
    }
}
