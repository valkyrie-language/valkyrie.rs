use crate::{
    helpers::Hir2Mir, structures::ValkyrieResource, ResolveState, ValkyrieClass, ValkyrieEnumeration, ValkyrieField,
    ValkyrieFlagation, ValkyrieMethod, ValkyrieSemanticNumber, ValkyrieUnite, ValkyrieUniteItem,
};
use indexmap::IndexMap;
use nyar_error::Result;
use valkyrie_ast::{
    ClassDeclaration, ClassTerm, EncodeDeclaration, FieldDeclaration, FlagDeclaration, FlagKind, FlagTerm, MethodDeclaration,
    UnionDeclaration, UnionTerm, VariantDeclaration,
};

impl Hir2Mir for UnionDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let name = store.register_item(&self.name);
        let mut output = ValkyrieUnite { unite_name: name, variants: Default::default() };
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
    type Output = ValkyrieUniteItem;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> nyar_error::Result<Self::Output> {
        let (variant_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;
        let mut output =
            ValkyrieUniteItem { variant_name, wasi_alias, type_alias: Default::default(), fields: Default::default() };
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

impl Hir2Mir for FlagDeclaration {
    type Output = ();
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        let name = store.register_item(&self.name);

        // let mut terms = vec![];

        for term in self.body.into_iter() {
            match term {
                FlagTerm::Macro(_) => {}
                FlagTerm::Encode(v) => {}
                FlagTerm::Method(_) => {}
            }
        }

        match self.kind {
            FlagKind::Enumerate => *store += ValkyrieEnumeration { enumeration_name: name, indexes: Default::default() },
            FlagKind::Flags => *store += ValkyrieFlagation { flags_name: name, flags: Default::default() },
        }

        Ok(())
    }
}

impl Hir2Mir for EncodeDeclaration {
    type Output = ValkyrieSemanticNumber;
    type Context = ();

    fn to_mir(self, store: &mut ResolveState, context: &Self::Context) -> Result<Self::Output> {
        Ok(ValkyrieSemanticNumber { number_name: self.name.name.clone() })
    }
}

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
