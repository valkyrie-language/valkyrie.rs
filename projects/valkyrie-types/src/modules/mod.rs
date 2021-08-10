use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    structures::ValkyrieClass,
    ValkyrieFunction, ValkyrieUnion,
};
use convert_case::{Case, Casing};
use im::{hashmap::Entry, HashMap};
use indexmap::IndexMap;
use nyar_error::{Failure, ForeignInterfaceError, NyarError, Result, SourceCache, SourceSpan, Success};
use nyar_wasm::{CanonicalWasi, DependentGraph, Identifier, WasiImport, WasiModule};
use std::{
    fmt::{Debug, Formatter},
    mem::take,
    path::Path,
    str::FromStr,
    sync::Arc,
};
use valkyrie_ast::{AnnotationNode, ArgumentTerm, IdentifierNode, NamespaceDeclaration, ProgramRoot, StatementKind};
use valkyrie_parser::{ProgramContext, StatementNode};

mod codegen;
mod display;
mod parser;

pub struct ValkyrieModule {}

/// Convert file to module
pub struct ResolveState {
    pub(crate) package: Arc<str>,
    /// The current namespace
    pub(crate) namespace: Vec<Arc<str>>,
    /// The document buffer
    pub(crate) document: String,
    /// Mapping local name to global name
    pub(crate) name_mapping: HashMap<Vec<Arc<str>>, ModuleImportsMap>,
    /// The declared items in file
    pub(crate) items: IndexMap<Identifier, ModuleItem>,
    /// Collect errors
    errors: Vec<NyarError>,
    /// Collect spread statements
    pub(crate) main_function: Vec<StatementNode>,
    sources: SourceCache,
}

#[derive(Clone, Default)]
pub struct ModuleImportsMap {
    using: HashMap<Identifier, Identifier>,
    local: HashMap<Identifier, Identifier>,
}

pub enum ModuleItem {
    Function(ValkyrieFunction),
    // Imported(ValkyrieSymbol),
    // Function(ValkyrieFunction),
    Structure(ValkyrieClass),
    Variant(ValkyrieUnion),
}

impl ResolveState {
    pub fn new<S: Into<Arc<str>>>(package: S) -> Self {
        Self {
            package: package.into(),
            namespace: vec![],
            document: "".to_string(),
            name_mapping: Default::default(),
            items: Default::default(),
            errors: vec![],
            main_function: vec![],
            sources: Default::default(),
        }
    }
}

impl ResolveState {
    pub fn reset_namespace(&mut self) {
        self.namespace = vec![];
    }
}

impl ResolveState {
    /// Get the full name path based on package name and namespace, then register the name to local namespace.
    pub fn register_item(&mut self, symbol: &IdentifierNode) -> Identifier {
        let key = Identifier { namespace: vec![], name: symbol.name.clone() };
        let value = Identifier { namespace: self.namespace.clone(), name: symbol.name.clone() };
        match self.name_mapping.entry(self.namespace.clone()) {
            Entry::Occupied(v) => {
                v.into_mut().local.insert(key, value.clone());
            }
            Entry::Vacant(v) => {
                let mut map = ModuleImportsMap::default();
                map.using.insert(key, value.clone());
                v.insert(map);
            }
        }
        value
    }
    pub fn get_foreign_module(
        &mut self,
        info: &AnnotationNode,
        kind: &'static str,
        hint: &'static str,
        keyword: SourceSpan,
    ) -> Option<(WasiModule, Arc<str>)> {
        let ffi = info.attributes.get("import")?;
        if !hint.is_empty() {
            if !info.modifiers.contains(hint) {
                self.push_error(ForeignInterfaceError::MissingForeignFlag { kind, hint, span: keyword });
            }
        }
        match ffi.get_ffi_modules() {
            Ok((module, name)) => match WasiModule::from_str(&module.text) {
                Ok(o) => return Some((o, name)),
                Err(e) => self.push_error(e.with_span(module.span.clone())),
            },
            Err(e) => self.push_error(e),
        }
        return None;
    }
    /// Get the full name path based on package name and namespace
    pub fn export_field(&self, symbol: &IdentifierNode, alias: &AnnotationNode) -> Result<(Arc<str>, Arc<str>)> {
        let wasi_alias = match alias.attributes.get("export").and_then(|x| x.arguments.terms.first()) {
            Some(s) => match s.value.as_text() {
                Some(s) => Arc::from(s.text.as_str()),
                None => Err(NyarError::custom("missing wasi alias"))?,
            },
            None => Arc::from(symbol.name.as_ref().to_case(Case::Kebab)),
        };
        Ok((symbol.name.clone(), wasi_alias))
    }

    /// Get the full name path based on package name and namespace
    pub fn wasi_import_module_name(&mut self, alias: &AnnotationNode, symbol: &IdentifierNode) -> Option<WasiImport> {
        let import = alias.attributes.get("import")?;
        let module = self.find_wasi_module(import.arguments.terms.get(0), import.span)?;
        let name: Arc<str> = match import.arguments.terms.get(1) {
            Some(term) => match term.value.as_text() {
                Some(node) => Arc::from(node.text.as_str()),
                None => {
                    self.push_error(ForeignInterfaceError::InvalidForeignName { span: term.span });
                    return None;
                }
            },
            None => Arc::from(symbol.name.as_ref().to_case(Case::Kebab)),
        };
        Some(WasiImport { module, name })
    }

    fn find_wasi_module(&mut self, term: Option<&ArgumentTerm>, span: SourceSpan) -> Option<WasiModule> {
        match term.and_then(|x| x.value.as_text()) {
            Some(text) => match WasiModule::from_str(&text.text) {
                Ok(o) => Some(o),
                Err(e) => {
                    self.push_error(e.with_span(text.span.clone()));
                    None
                }
            },
            None => {
                self.push_error(ForeignInterfaceError::InvalidForeignModule { span });
                None
            }
        }
    }
}
