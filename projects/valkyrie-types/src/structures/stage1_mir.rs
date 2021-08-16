use super::*;

impl Hir2Mir for ClassDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        let symbol = store.register_item(&self.name);
        let mut methods = IndexMap::default();
        let mut fields = IndexMap::default();

        for x in self.terms {
            match x {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(store, &())?;
                    match fields.insert(field.field_name.clone(), field) {
                        Some(s) => {
                            unimplemented!()
                        }
                        None => {}
                    }
                }
                ClassTerm::Method(v) => {
                    let method = v.to_mir(store, &())?;
                    match methods.insert(method.method_name.clone(), method) {
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

        match store.wasi_import_module_name(&self.annotations, &self.name) {
            Some(wasi_import) => {
                *store += ValkyrieResource { resource_name: symbol, wasi_import, methods };
            }
            None => *store += ValkyrieClass { class_name: symbol, fields, methods },
        }

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
