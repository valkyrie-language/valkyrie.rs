use super::*;

impl Hir2Mir for UnionDeclaration {
    type Output = ();
    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output> {
        let name = ctx.register_item(&self.name);
        let mut output = ValkyrieUnion { union_name: name, variants: Default::default() };
        for item in self.body {
            match item {
                UnionTerm::Macro(_) => {
                    todo!()
                }
                UnionTerm::Variant(v) => {
                    let variant = v.to_mir(ctx)?;
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
        *ctx += output;
        Ok(())
    }
}

impl Hir2Mir for VariantDeclaration {
    type Output = ValkyrieUnionItem;
    fn to_mir(self, ctx: &mut ResolveContext) -> nyar_error::Result<Self::Output> {
        let (variant_name, wasi_alias) = ctx.export_field(&self.name, &self.annotations)?;
        let mut output =
            ValkyrieUnionItem { variant_name, wasi_alias, type_alias: Default::default(), fields: Default::default() };
        for item in self.body {
            match item {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(ctx)?;
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
