use super::*;

impl Hir2Mir for ClassDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        let symbol = store.register_item(&self.name);
        let mut class = ValkyrieClass::new(symbol);
        class.wasi_import = store.wasi_import_module_name(&self.annotations, &self.name);
        for x in self.terms {
            match x {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(store, &())?;
                    match class.fields.insert(field.field_name.clone(), field) {
                        Some(s) => {
                            unimplemented!()
                        }
                        None => {}
                    }
                }
                ClassTerm::Method(v) => {
                    let method = v.to_mir(store, &())?;
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
        *store += class;

        Ok(())
    }
}

impl Hir2Mir for FieldDeclaration {
    type Output = ValkyrieField;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        let (field_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;

        Ok(ValkyrieField { field_name, wasi_alias })
    }
}
impl Hir2Mir for MethodDeclaration {
    type Output = ValkyrieMethod;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        let wasi_import = store.wasi_import_module_name(&self.annotations, &self.name);
        Ok(ValkyrieMethod { method_name: self.name.name.clone(), wasi_import, wasi_export: None })
    }
}
