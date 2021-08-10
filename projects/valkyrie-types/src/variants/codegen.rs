use super::*;

impl Hir2Mir for UnionDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let name = store.register_item(&self.name);
        let mut output = ValkyrieUnion { union_name: name, variants: Default::default() };
        for item in self.body {
            match item {
                UnionTerm::Macro(_) => {
                    todo!()
                }
                UnionTerm::Variant(v) => {
                    let variant = v.to_mir(store, &())?;
                    match output.variants.insert(variant.variant_name.clone(), variant) {
                        Some(_) => {
                            panic!("dup variant")
                        }
                        None => {}
                    }
                }
                UnionTerm::Method(_) => {
                    todo!()
                }
            }
        }
        *store += output;
        Ok(())
    }
}

impl Hir2Mir for VariantDeclaration {
    type Output = ValkyrieUnionItem;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let (variant_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;
        let mut output =
            ValkyrieUnionItem { variant_name, wasi_alias, type_alias: Default::default(), fields: Default::default() };
        for item in self.body {
            match item {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(store, &())?;
                    output.fields.insert(field.field_name.clone(), field);
                }
                ClassTerm::Method(_) => {
                    todo!()
                }
                ClassTerm::Domain(_) => {
                    todo!()
                }
            }
        }
        Ok(output)
    }
}
