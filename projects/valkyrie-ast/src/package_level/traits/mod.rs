use super::*;

mod display;

// pub enum TraitKind {
//     Trait,
// }

/// `trait name: Debug {}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitDeclaration {
    /// The name of the trait
    pub name: IdentifierNode,
    /// the needed fields(zero parameter method, get + set)
    pub terms: Vec<ClassTerm>,
}

/// `extends path::A: Debug {}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendsStatement {
    /// The additional methods
    pub body: Vec<ClassTerm>,
}

// pub enum TraitKind {
//     Interface,
// }
