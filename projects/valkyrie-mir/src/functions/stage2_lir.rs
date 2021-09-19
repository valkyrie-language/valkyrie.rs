use super::*;
use std::mem::transmute;
use valkyrie_lir::{WasiFunctionBody, WasiParameter, WasiType};
use valkyrie_types::SyntaxError;

impl Mir2Lir for ValkyrieImportFunction {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        let mut function = WasiFunction::external(&self.wasi_import.module, &self.wasi_import.name, &self.function_name);
        for param in self.signature.positional.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }
        for param in self.signature.mixed.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }
        for param in self.signature.named.values() {
            function.inputs.push(param.to_lir(graph, context)?)
        }

        *graph += function;
        Ok(())
    }
}

impl Mir2Lir for ValkyrieNativeFunction {
    type Output = ();
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        for (id, f) in self.overloads.iter() {
            let name = if id.eq(&0.0) {
                self.function_name.clone()
            }
            else {
                self.function_name.join(Identifier::new(&format!("0x{:X}", unsafe { transmute::<f64, u64>(id.into_inner()) })))
            };
            let inputs = f.signature.to_lir(graph, context)?;

            if f.body.assembly.is_empty() {
                *graph +=
                    WasiFunction { symbol: name, inputs, output: vec![], body: WasiFunctionBody::Native { bytecodes: vec![] } };
            }
            else {
                // *graph += WasiFunction {
                //     symbol: name,
                //     inputs,
                //     output: vec![],
                //     body: WasiFunctionBody::Assembly { string: f.body.assembly.clone() },
                // };
            }
        }

        Ok(())
    }
}

impl Mir2Lir for FunctionSignature {
    type Output = Vec<WasiParameter>;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        let mut outs = Vec::with_capacity(16);
        for param in self.positional.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        for param in self.mixed.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        for param in self.named.values() {
            outs.push(param.to_lir(graph, context)?)
        }
        Ok(outs)
    }
}

impl Mir2Lir for FunctionParameter {
    type Output = WasiParameter;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        let typing = self.r#type.to_lir(graph, context)?;
        Ok(WasiParameter { name: self.name.clone(), wasi_name: self.name.clone(), r#type: typing })
    }
}

impl Mir2Lir for ValkyrieType {
    type Output = WasiType;
    type Context<'a> = &'a ResolveContext;

    fn to_lir<'a>(&self, graph: &mut DependentGraph, context: Self::Context<'a>) -> valkyrie_types::Result<Self::Output> {
        let wasi_ty = match self {
            ValkyrieType::Boolean => WasiType::Boolean,
            ValkyrieType::Integer { bits } => match bits {
                8 => WasiType::Integer8 { signed: true },
                16 => WasiType::Integer16 { signed: true },
                32 => WasiType::Integer32 { signed: true },
                64 => WasiType::Integer64 { signed: true },
                bits => Err(SyntaxError::new(format!("Unsupported integer type with {} bits", bits)))?,
            },
            ValkyrieType::Unsigned { bits } => match bits {
                8 => WasiType::Integer8 { signed: false },
                16 => WasiType::Integer16 { signed: false },
                32 => WasiType::Integer32 { signed: false },
                64 => WasiType::Integer64 { signed: false },
                bits => Err(SyntaxError::new(format!("Unsupported unsigned type with {} bits", bits)))?,
            },
            ValkyrieType::Float { bits } => match bits {
                32 => WasiType::Float32,
                64 => WasiType::Float64,
                bits => Err(SyntaxError::new(format!("Unsupported float type with {} bits", bits)))?,
            },
            ValkyrieType::Unicode => WasiType::Unicode,
            ValkyrieType::Unsolved(v) => Err(SyntaxError::new(format!("{v}")))?,
        };
        Ok(wasi_ty)
    }
}
