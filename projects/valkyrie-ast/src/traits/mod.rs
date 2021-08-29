use crate::{FieldDeclaration, IdentifierNode, NamePathNode, ProgramRoot, StatementKind, TraitDeclaration, TraitTerm};
use alloc::{boxed::Box, string::String, vec::Vec};

// This token can call references
pub enum DefinitionQuery {
    Trait(Vec<String>),
}

pub struct DefinitionContext {
    pub names: Vec<String>,
}

pub trait ReferenceCaller {}

#[allow(unused_variables)]
pub trait ElementQuery {
    fn query_definition_provider(&self, cursor: usize) -> Option<DefinitionQuery> {
        None
    }
}

impl<T> ElementQuery for Vec<T>
where
    T: ElementQuery,
{
    fn query_definition_provider(&self, cursor: usize) -> Option<&dyn DefinitionQuery> {
        for item in self.iter() {
            match item.query_definition_provider(cursor) {
                Some(s) => return Some(s),
                None => continue,
            }
        }
        return None;
    }
}

impl ElementQuery for ProgramRoot {
    fn query_definition_provider(&self, offset: usize) -> Option<&dyn DefinitionQuery> {
        self.statements.query_definition_provider(offset)
    }
}

impl ElementQuery for StatementKind {
    fn query_definition_provider(&self, offset: usize) -> Option<&dyn DefinitionQuery> {
        match self {
            StatementKind::Nothing => None,
            StatementKind::Document(_) => None,
            StatementKind::Annotation(_) => None,
            StatementKind::Namespace(_) => None,
            StatementKind::Import(_) => None,
            StatementKind::Class(_) => None,
            StatementKind::Union(_) => None,
            StatementKind::Enumerate(_) => None,
            StatementKind::Trait(v) => v.query_definition_provider(offset),
            StatementKind::Extends(_) => None,
            StatementKind::Function(_) => None,
            StatementKind::Variable(_) => None,
            StatementKind::Guard(_) => None,
            StatementKind::While(_) => None,
            StatementKind::For(_) => None,
            StatementKind::Control(_) => None,
            StatementKind::Expression(_) => None,
        }
    }
}

impl ElementQuery for TraitDeclaration {
    fn query_definition_provider(&self, offset: usize) -> Option<&dyn DefinitionQuery> {
        if self.span.contains(&offset) {
            // if cursor on trait elements
            match self.body.query_definition_provider(offset) {
                Some(s) => Some(s),
                None => Some(self),
            }
        }
        else {
            None
        }
    }
}

impl DefinitionQuery for TraitDeclaration {}

impl ElementQuery for TraitTerm {
    fn query_definition_provider(&self, offset: usize) -> Option<&dyn DefinitionQuery> {
        match self {
            TraitTerm::Macro(_) => None,
            TraitTerm::Field(v) => v.query_definition_provider(offset),
            TraitTerm::Method(_) => None,
        }
    }
}

impl ElementQuery for FieldDeclaration {
    fn query_definition_provider(&self, offset: usize) -> Option<&dyn DefinitionQuery> {
        if self.span.contains(&offset) { Some(self) } else { None }
    }
}

impl DefinitionQuery for FieldDeclaration {}
