use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    structures::{ValkyrieClass, ValkyrieResource},
    ValkyrieEnumeration, ValkyrieFlagation, ValkyrieImportFunction, ValkyrieNativeFunction, ValkyriePrimitive, ValkyrieUnite,
};
use im::{hashmap::Entry, HashMap};
use indexmap::IndexMap;
use std::{
    fmt::{Debug, Formatter},
    mem::take,
    path::Path,
    str::FromStr,
    sync::Arc,
};
use valkyrie_ast::{AnnotationNode, ArgumentTerm, IdentifierNode, ProgramRoot};
use valkyrie_lir::{CanonicalWasi, DependentGraph, WasiImport, WasiModule, WasmIdentifier};
use valkyrie_parser::ProgramContext;
use valkyrie_types::{Failure, ForeignInterfaceError, Identifier, NyarError, Result, SourceCache, SourceSpan, Success};

mod codegen;
mod display;
mod parser;

pub struct ValkyrieModule {}

/// Convert file to module
pub struct ResolveContext {
    pub(crate) package: Identifier,
    /// The current namespace
    pub(crate) namespace: Vec<Identifier>,
    /// The document buffer
    pub(crate) document: String,
    /// Mapping local name to global name
    pub(crate) name_mapping: HashMap<Vec<Identifier>, ModuleImportsMap>,
    /// The declared items in file
    pub(crate) items: IndexMap<WasmIdentifier, NamespaceItem>,
    /// Collect errors
    errors: Vec<NyarError>,
    /// Collect spread statements
    pub(crate) main_function: Vec<ValkyrieStatement>,
    sources: SourceCache,
}

#[derive(Debug)]
pub enum ValkyrieStatement {}

#[derive(Clone, Default)]
pub struct ModuleImportsMap {
    using: HashMap<WasmIdentifier, WasmIdentifier>,
    local: HashMap<WasmIdentifier, WasmIdentifier>,
}

pub enum NamespaceItem {
    /// A unresolved symbol
    Unknown(WasmIdentifier),
    Resource(ValkyrieResource),
    Structure(ValkyrieClass),
    Primitive(ValkyriePrimitive),
    Flags(ValkyrieFlagation),
    Enums(ValkyrieEnumeration),
    Variant(ValkyrieUnite),
    External(ValkyrieImportFunction),
    Function(ValkyrieNativeFunction),
}

impl ResolveContext {
    pub fn new(package: Identifier) -> Self {
        Self {
            package,
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

impl ResolveContext {
    pub fn reset_namespace(&mut self) {
        self.namespace = vec![];
    }
}

impl ResolveContext {
    /// Get the full name path based on package name and namespace, then register the name to local namespace.
    pub fn register_item(&mut self, symbol: &IdentifierNode) -> WasmIdentifier {
        let key = WasmIdentifier { namespace: vec![], name: symbol.name };
        let value = WasmIdentifier { namespace: self.namespace.clone(), name: symbol.name };
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
    ) -> Option<(WasiModule, Identifier)> {
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
    pub fn export_field(&self, symbol: &IdentifierNode, alias: &AnnotationNode) -> Result<(Identifier, Identifier)> {
        let wasi_alias = match alias.attributes.get("export").and_then(|x| x.arguments.terms.first()) {
            Some(s) => match s.value.as_text() {
                Some(s) => Identifier::new(s.text.as_str()),
                None => Err(NyarError::custom("missing wasi alias"))?,
            },
            None => symbol.name.to_kebab_case(),
        };
        Ok((symbol.name, wasi_alias))
    }

    /// Get the full name path based on package name and namespace
    pub fn wasi_import_module_name(&mut self, alias: &AnnotationNode, symbol: &IdentifierNode) -> Option<WasiImport> {
        let import = alias.attributes.get("import")?;
        let module = self.find_wasi_module(import.arguments.terms.get(0), import.span)?;
        let name: Identifier = match import.arguments.terms.get(1) {
            Some(term) => match term.value.as_text() {
                Some(node) => Identifier::new(node.text.as_str()),
                None => {
                    self.push_error(ForeignInterfaceError::InvalidForeignName { span: term.span });
                    return None;
                }
            },
            None => symbol.name.to_kebab_case(),
        };
        Some(WasiImport { module, name })
    }
    pub fn find_wasi_alias(&self, alias: &AnnotationNode, symbol: &IdentifierNode) -> Identifier {
        match self.try_wasi_alias(alias) {
            Some(s) => Identifier::new(s),
            None => symbol.name.to_kebab_case(),
        }
    }
    fn try_wasi_alias<'a>(&self, alias: &'a AnnotationNode) -> Option<&'a str> {
        let import = alias.attributes.get("export")?;
        let first = import.arguments.terms.get(0)?;
        let text = first.value.as_text()?;
        Some(&text.text)
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
