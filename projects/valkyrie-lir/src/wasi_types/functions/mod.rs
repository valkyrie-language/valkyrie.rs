use std::{
    fmt::{Display, Formatter, Write},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_types::Identifier;
use crate::{
    WasiModule, WasiType, WasmIdentifier, WastEncoder,
    dag::DependentGraph,
    helpers::{DependenciesTrace, TypeReferenceInput},
    operations::WasiInstruction,
};

mod arithmetic;
mod display;

/// The type of external WASI function
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WasiFunction {
    /// The symbol of the function in source language
    pub symbol: WasmIdentifier,
    /// The input parameters of the function
    pub inputs: Vec<WasiParameter>,
    /// The output parameter of the function
    pub output: Vec<WasiParameter>,
    pub body: WasiFunctionBody,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WasiFunctionBody {
    External {
        /// The external module name registered in WASI host
        wasi_module: WasiModule,
        /// The external function name registered in WASI host
        wasi_name: Identifier,
    },
    Native {
        bytecodes: Vec<WasiInstruction>,
    },
    Assembly {
        text: String,
    },
}

/// The type of WASI parameter
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WasiParameter {
    /// The name of the parameter in source language
    pub name: Identifier,
    /// The name of the parameter in WASI host
    pub wasi_name: Identifier,
    /// The type of the parameter
    pub r#type: WasiType,
}

impl WasiFunction {
    /// Create a new external function type with the given symbol and WASI module
    pub fn external(wasi_module: &WasiModule, wasi_name: &Identifier, name: &WasmIdentifier) -> Self {
        Self {
            symbol: name.clone(),
            inputs: vec![],
            output: vec![],
            body: WasiFunctionBody::External { wasi_module: wasi_module.clone(), wasi_name: wasi_name.clone() },
        }
    }

    pub fn need_heap(&self) -> bool {
        for x in self.inputs.iter() {
            if x.r#type.is_heap_type() {
                return true;
            }
        }
        for x in self.output.iter() {
            if x.r#type.is_heap_type() {
                return true;
            }
        }
        return false;
    }
    pub fn need_encoding(&self) -> Option<&'static str> {
        for i in self.inputs.iter() {
            match i.r#type {
                _ => {}
            }
        }

        None
    }
}

impl WasiParameter {
    /// Create a new WASI parameter with the given name and type
    pub fn new<T>(name: Identifier, r#type: T) -> Self
    where
        T: Into<WasiType>,
    {
        Self { name, wasi_name: name, r#type: r#type.into() }
    }
}

impl DependenciesTrace for WasiFunction {
    fn define_language_types(&self, dict: &mut DependentGraph) {
        dict.types.insert(self.symbol.clone(), WasiType::Function(Box::new(self.clone())));
    }

    fn collect_wasi_types<'a, 'i>(&'a self, dict: &'i DependentGraph, collected: &mut Vec<&'i WasiType>)
    where
        'a: 'i,
    {
        self.inputs.iter().for_each(|input| input.r#type.collect_wasi_types(dict, collected));
        self.output.iter().for_each(|output| output.r#type.collect_wasi_types(dict, collected));
    }
}
