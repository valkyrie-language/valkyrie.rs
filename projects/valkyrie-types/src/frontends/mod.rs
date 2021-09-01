use crate::{
    functions::{FunctionBody, FunctionInstance, FunctionParameter},
    helpers::{AsIdentifier, Hir2Mir},
    structures::ValkyrieResource,
    ModuleItem, ResolveState, ValkyrieClass, ValkyrieEnumeration, ValkyrieField, ValkyrieFlagation, ValkyrieFrom,
    ValkyrieImportFunction, ValkyrieMethod, ValkyrieNativeFunction, ValkyrieSemanticNumber, ValkyrieType, ValkyrieUnite,
    ValkyrieVariant,
};
use indexmap::IndexMap;
use nyar_error::Result;
use nyar_wasm::Identifier;
use ordered_float::NotNan;
use std::{collections::BTreeMap, sync::Arc};
use valkyrie_ast::{
    ClassDeclaration, ClassTerm, EncodeDeclaration, ExpressionKind, FieldDeclaration, FlagTerm, FunctionDeclaration,
    ImplementsStatement, MethodDeclaration, NamespaceDeclaration, ParameterTerm, ProgramRoot, SemanticKind, SemanticNumber,
    StatementKind, TraitDeclaration, TraitTerm, UnionDeclaration, UnionTerm, VariantDeclaration,
};

impl Hir2Mir for ProgramRoot {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        for statement in self.statements {
            statement.to_mir(store, ())?
        }
        Ok(())
    }
}

impl Hir2Mir for StatementKind {
    type Output = ();
    type Context<'a> = ();

    fn to_mir(self, store: &mut ResolveState, context: Self::Context<'_>) -> Result<Self::Output> {
        match self {
            Self::Nothing => {}
            Self::Document(_) => {
                todo!()
            }
            Self::Annotation(_) => {
                todo!()
            }
            Self::Namespace(v) => v.to_mir(store, ())?,
            Self::Import(i) => {}
            Self::Class(v) => v.to_mir(store, ())?,
            Self::Union(v) => v.to_mir(store, ())?,
            Self::Enumerate(v) => v.to_mir(store, ())?,
            Self::Trait(v) => v.to_mir(store, ())?,
            Self::Extends(v) => v.to_mir(store, ())?,
            Self::Function(v) => v.to_mir(store, ())?,
            Self::Variable(_) => {}
            Self::Guard(_) => {
                todo!()
            }
            Self::Loop(_) => {
                todo!()
            }
            Self::While(_) => {
                todo!()
            }
            Self::Until(_) => {
                todo!()
            }
            Self::Each(_) => {
                todo!()
            }
            Self::Control(_) => {
                todo!()
            }
            Self::Expression(_) => {
                todo!()
            }
        }
        Ok(())
    }
}

impl Hir2Mir for NamespaceDeclaration {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        store.namespace.clear();
        match self.path.path.as_slice() {
            // clear current namespace
            [head] if head.name.as_ref().eq("_") => {}
            [head, rest @ ..] => {
                match head.name.as_ref().eq("package") {
                    true => store.namespace.push(store.package.clone()),
                    false => store.namespace.push(head.name.clone()),
                }
                for x in rest {
                    store.namespace.push(x.name.clone())
                }
            }
            _ => {}
        }
        Ok(())
    }
}
impl Hir2Mir for TraitDeclaration {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        todo!()
    }
}

impl Hir2Mir for ImplementsStatement {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        for x in self.annotations.derives() {
            if x.path.last().unwrap().name.as_ref().eq("TypeCast") {
                let id = self.target.as_identifier();
                match store.items.get_mut(&id) {
                    Some(ModuleItem::Structure(class)) => {
                        for item in &self.body {
                            match item {
                                TraitTerm::Macro(_) => {
                                    todo!()
                                }
                                TraitTerm::Field(_) => {
                                    todo!()
                                }
                                TraitTerm::Method(f) => match f.name.name.as_ref() {
                                    "from" => class.register_from(&f)?,
                                    "into" => {}
                                    "try_from" => {}
                                    "try_into" => {}
                                    _ => {
                                        todo!()
                                    }
                                },
                            }
                        }
                    }
                    Some(s) => todo!(),
                    None => {
                        todo!()
                    }
                }
            }
        }

        Ok(())
    }
}

impl Hir2Mir for ClassDeclaration {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let symbol = store.register_item(&self.name);
        let mut imports = IndexMap::default();
        let mut methods = IndexMap::default();
        let mut fields = IndexMap::default();

        for x in self.terms {
            match x {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(store, ())?;
                    match fields.insert(field.field_name.clone(), field) {
                        Some(s) => {
                            unimplemented!()
                        }
                        None => {}
                    }
                }
                ClassTerm::Method(v) => {
                    let method = v.to_mir(store, &symbol)?;
                    match methods.insert(method.method_name.clone(), method) {
                        Some(s) => {}
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
                *store += ValkyrieResource { resource_name: symbol, wasi_import, imports };
            }
            None => {
                *store +=
                    ValkyrieClass { class_name: symbol, primitive: None, fields, imports, methods, from: vec![], into: vec![] }
            }
        }

        Ok(())
    }
}

impl Hir2Mir for MethodDeclaration {
    type Output = ValkyrieMethod;
    type Context<'a> = &'a Identifier;

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let (field_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;

        Ok(ValkyrieMethod { method_name: field_name, wasi_alias, overloads: Default::default() })
    }
}

impl Hir2Mir for FieldDeclaration {
    type Output = ValkyrieField;
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let (field_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;

        Ok(ValkyrieField { field_name, wasi_alias })
    }
}

impl Hir2Mir for UnionDeclaration {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let name = store.register_item(&self.name);
        let mut output = ValkyrieUnite { unite_name: name, variants: Default::default(), source: Default::default() };
        for item in self.body {
            match item {
                UnionTerm::Macro(_) => {
                    todo!()
                }
                UnionTerm::Variant(v) => {
                    let variant = v.to_mir(store, ())?;
                    match output.variants.insert(variant.variant_name.clone(), variant) {
                        Some(old) => {
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
    type Output = ValkyrieVariant;
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let (variant_name, wasi_alias) = store.export_field(&self.name, &self.annotations)?;
        let mut output = ValkyrieVariant {
            variant_name,
            wasi_alias,
            type_alias: Default::default(),
            fields: Default::default(),
            source: Default::default(),
        };
        for item in self.body {
            match item {
                ClassTerm::Macro(_) => {
                    todo!()
                }
                ClassTerm::Field(v) => {
                    let field = v.to_mir(store, ())?;
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

impl Hir2Mir for SemanticNumber {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let name = store.register_item(&self.name);

        let mut terms = IndexMap::default();

        for term in self.body.into_iter() {
            match term {
                FlagTerm::Macro(_) => {}
                FlagTerm::Encode(v) => {
                    let term = v.to_mir(store, ())?;
                    terms.insert(term.number_name.clone(), term);
                }
                FlagTerm::Method(_) => {}
            }
        }

        match self.kind {
            SemanticKind::Enumerate => *store += ValkyrieEnumeration { enumeration_name: name, enumerations: terms },
            SemanticKind::Flags => *store += ValkyrieFlagation { flags_name: name, flags: terms },
        }

        Ok(())
    }
}

impl Hir2Mir for EncodeDeclaration {
    type Output = ValkyrieSemanticNumber;
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> Result<Self::Output> {
        let wasm_alias = store.find_wasi_alias(&self.annotations, &self.name);
        Ok(ValkyrieSemanticNumber { number_name: self.name.name.clone(), wasm_alias })
    }
}

impl Hir2Mir for FunctionDeclaration {
    type Output = ();
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
        let function_name = store.register_item(&self.name);
        let mut instance = FunctionInstance::default();

        for parameter in self.parameters.positional {
            match parameter.to_mir(store, ()) {
                Ok(o) => {
                    instance.signature.positional.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.mixed {
            match parameter.to_mir(store, ()) {
                Ok(o) => {
                    instance.signature.mixed.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        for parameter in self.parameters.named {
            match parameter.to_mir(store, ()) {
                Ok(o) => {
                    instance.signature.named.insert(o.name.clone(), o);
                }
                Err(e) => store.push_error(e),
            }
        }
        match store.wasi_import_module_name(&self.annotations, &self.name) {
            Some(wasi_import) => {
                *store += ValkyrieImportFunction { function_name, wasi_import, signature: instance.signature };
            }
            None => {
                let mut overloads = BTreeMap::default();
                unsafe {
                    overloads.insert(NotNan::new_unchecked(0.0), instance);
                }
                *store += ValkyrieNativeFunction { function_name, wasi_export: None, overloads };
            }
        }

        return Ok(());
    }
}

impl Hir2Mir for ParameterTerm {
    type Output = FunctionParameter;
    type Context<'a> = ();

    fn to_mir<'a>(self, store: &mut ResolveState, context: Self::Context<'a>) -> nyar_error::Result<Self::Output> {
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
