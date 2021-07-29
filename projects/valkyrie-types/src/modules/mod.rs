use crate::{
    values::symbols::AsSymbol, FileCache, FileID, ValkyrieExternalFunction, ValkyrieFunction, ValkyrieStructure, ValkyrieSymbol,
};
use indexmap::IndexMap;
use nyar_error::{Failure, NyarError, Result, Success};
use std::{
    fmt::{Debug, Formatter},
    mem::take,
};
use valkyrie_ast::{NamespaceDeclaration, ProgramRoot, StatementKind};
use valkyrie_parser::{ProgramContext, StatementNode};

pub struct ValkyrieModule {}

/// Convert file to module
#[derive(Debug, Default)]
pub struct ModuleResolver {
    /// The declared namespace
    pub(crate) namespace: Option<ValkyrieSymbol>,
    /// main function of the file
    pub(crate) main: Vec<StatementNode>,
    /// The declared items in file
    pub(crate) items: IndexMap<String, ModuleItem>,
    /// Collect errors
    pub(crate) errors: Vec<NyarError>,
}

pub enum ModuleItem {
    External(ValkyrieExternalFunction),
    Imported(ValkyrieSymbol),
    Function(ValkyrieFunction),
    Structure(ValkyrieStructure),
}

impl Debug for ModuleItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::External(v) => Debug::fmt(v, f),
            Self::Imported(v) => Debug::fmt(v, f),
            Self::Structure(v) => Debug::fmt(v, f),
            Self::Function(v) => Debug::fmt(v, f),
        }
    }
}

pub(crate) trait Hir2Mir {
    type Output = ();
    fn to_mir(self, ctx: &mut ModuleResolver) -> Result<Self::Output>;
}

impl Hir2Mir for ProgramRoot {
    fn to_mir(self, ctx: &mut ModuleResolver) -> Result<Self::Output> {
        for statement in self.statements {
            statement.to_mir(ctx)?
        }
        Ok(())
    }
}

impl Hir2Mir for StatementKind {
    fn to_mir(self, ctx: &mut ModuleResolver) -> Result<Self::Output> {
        match self {
            Self::Nothing => {}
            Self::Document(_) => {}
            Self::Annotation(_) => {}
            Self::Namespace(v) => v.to_mir(ctx)?,
            Self::Import(_) => {}
            Self::Class(v) => v.to_mir(ctx)?,
            Self::Union(_) => {}
            Self::Enumerate(_) => {}
            Self::Trait(_) => {}
            Self::Extends(_) => {}
            Self::Function(f) => f.to_mir(ctx)?,
            Self::Variable(_) => {}
            Self::Guard(_) => {}
            Self::While(_) => {}
            Self::For(_) => {}
            Self::Control(_) => {}
            Self::Expression(_) => {}
        }
        Ok(())
    }
}

impl Hir2Mir for NamespaceDeclaration {
    fn to_mir(self, ctx: &mut ModuleResolver) -> Result<Self::Output> {
        ctx.namespace = Some(self.path.as_symbol());
        Ok(())
    }
}
impl ModuleResolver {
    pub fn parse(&mut self, file: FileID, cache: &mut FileCache) -> Vec<NyarError> {
        let root = ProgramContext { file }.parse(cache);
        let mut errors = vec![];
        match root {
            Success { value, diagnostics } => {
                errors.extend(diagnostics);
                errors.extend(self.visit(value))
            }
            Failure { fatal, diagnostics } => {
                errors.extend(diagnostics);
                errors.extend_one(fatal);
            }
        }
        errors
    }

    pub fn visit(&mut self, root: ProgramRoot) -> Vec<NyarError> {
        let progress = root.to_mir(self);
        let mut errors = take(&mut self.errors);
        match progress {
            Ok(_) => {}
            Err(e) => errors.push(e),
        }
        errors
    }
}
