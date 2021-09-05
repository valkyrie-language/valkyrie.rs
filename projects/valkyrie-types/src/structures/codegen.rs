use super::*;
use crate::helpers::AsIdentifier;
use nyar_error::NyarError;
use nyar_wasm::{
    WasiFunction, WasiFunctionBody, WasiOwnership, WasiParameter, WasiRecordField, WasiRecordType, WasiType, WasiTypeReference,
};

impl Mir2Lir for ValkyrieResource {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        // for method in self.methods.values() {
        //     method.to_lir(graph, &self.resource_name)?
        // }
        *graph += WasiResource {
            symbol: self.resource_name.clone(),
            wasi_module: self.wasi_import.module.clone(),
            wasi_name: self.wasi_import.name.clone(),
        };
        Ok(())
    }
}
impl Mir2Lir for ValkyrieClass {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        for method in self.methods.values() {
            method.to_lir(graph, context)?
        }
        for from in self.from.iter() {
            from.to_lir(graph, &self.class_name)?
        }
        let mut fields = IndexMap::default();
        for (key, field) in self.fields.iter() {
            match field.to_lir(graph, context) {
                Ok(field) => {
                    fields.insert(key.clone(), field);
                }
                Err(e) => {}
            }
        }
        match &self.primitive {
            Some(s) => {}
            None => {
                *graph += WasiRecordType { symbol: self.class_name.clone(), wasi_name: "".to_string(), fields };
            }
        }
        Ok(())
    }
}

impl Mir2Lir for ValkyriePrimitive {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        for method in self.methods.values() {
            method.to_lir(graph, context)?
        }
        for from in self.from.iter() {
            from.to_lir(graph, &self.primitive_name)?
        }
        Ok(())
    }
}

impl Mir2Lir for ValkyrieField {
    type Output = WasiRecordField;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: &ResolveContext) -> Result<Self::Output> {
        Ok(WasiRecordField {
            name: self.field_name.clone(),
            wasi_name: self.wasi_alias.clone(),
            r#type: WasiType::Boolean,
            default_value: None,
        })
    }
}

impl Mir2Lir for ValkyrieMethod {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        Ok(())
    }
}
impl Mir2Lir for ValkyrieFrom {
    type Output = ();
    type Context<'a> = &'a Identifier;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> Result<Self::Output> {
        *graph += WasiFunction {
            symbol: context.join("from"),
            inputs: vec![WasiParameter::new(
                "value",
                WasiType::TypeHandler(WasiTypeReference { symbol: self.from.clone(), owner: WasiOwnership::Normal }),
            )],
            output: vec![WasiParameter::new(
                "",
                WasiType::TypeHandler(WasiTypeReference { symbol: context.clone(), owner: WasiOwnership::Normal }),
            )],
            body: WasiFunctionBody::Assembly { text: self.action.assembly.clone() },
        };
        Ok(())
    }
}
impl ValkyrieClass {
    pub fn register_from(&mut self, method: &MethodDeclaration) -> Result<()> {
        let implicit = if method.annotations.modifiers.contains("explicit") {
            false
        }
        else if method.annotations.modifiers.contains("implicit") {
            true
        }
        else {
            return Err(NyarError::syntax_error("must one of `implicit`, `explicit`", method.name.span));
        };
        let body = match method.as_assembly()? {
            Some(s) => FunctionBody { assembly: s.text },
            None => FunctionBody { assembly: "".to_string() },
        };
        match method.parameters.mixed.first() {
            Some(parameter) => match parameter.bound.as_ref().and_then(|x| x.as_symbol()) {
                Some(ty) => self.from.push(ValkyrieFrom { from: ty.as_identifier(), implicit, action: body, exception: None }),
                None => return Err(NyarError::syntax_error("missing `value` type", parameter.key.span)),
            },
            None => return Err(NyarError::syntax_error("missing `value` parameter", method.name.span)),
        }
        Ok(())
    }
}
