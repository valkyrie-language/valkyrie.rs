//! Unified parameter matching entry points for the current `HIR` layer.
//!
//! This module centralizes the settled matching categories used by overload
//! resolution and future call checking:
//! - nominal parameter matching
//! - named trait satisfaction
//! - anonymous row requirement satisfaction

#![allow(missing_docs)]

use std::collections::BTreeMap;

use crate::hir::{
    nominal::{NominalModuleError, NominalModuleView},
    row::{RowMethodSignature, RowRequirement, RowRequirementError},
    trait_system::{NamedTraitWitness, TraitModuleError, TraitModuleView},
};
use valkyrie_types::{
    hir::{HirModule, HirStruct, RowType, TraitObject, ValkyrieType},
    Identifier,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRelationDiagnosticSeed {
    IncompatibleTypes { actual: ValkyrieType, expected: ValkyrieType },
    UnsupportedActualType { actual: ValkyrieType },
    UnsupportedExpectedType { expected: ValkyrieType },
    Nominal { error: NominalModuleError },
    Trait { error: TraitModuleError },
    Row { errors: Vec<RowRequirementError> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterMatchResult {
    NominalExact,
    NominalSubtype { distance: usize },
    Trait { witness: NamedTraitWitness },
    Row,
    NoMatch { diagnostic_seed: TypeRelationDiagnosticSeed },
}

#[derive(Debug, Clone)]
pub struct TypeRelationContext {
    nominal: NominalModuleView,
    traits: TraitModuleView,
    structs: BTreeMap<Identifier, HirStruct>,
}

impl TypeRelationContext {
    pub fn from_module(module: &HirModule) -> Self {
        Self {
            nominal: NominalModuleView::from_module(module),
            traits: TraitModuleView::from_module(module),
            structs: module.structs.iter().map(|item| (item.name.clone(), item.clone())).collect(),
        }
    }

    pub fn match_parameter(&self, actual: &ValkyrieType, expected: &ValkyrieType) -> ParameterMatchResult {
        if trivially_compatible(actual, expected) {
            return ParameterMatchResult::NominalExact;
        }

        match expected {
            ValkyrieType::Named(expected_name) => {
                if let Some(actual_name) = named_type_name(actual) {
                    let nominal = self.match_nominal_parameter(&actual_name, expected_name);
                    match &nominal {
                        ParameterMatchResult::NominalExact | ParameterMatchResult::NominalSubtype { .. } => return nominal,
                        ParameterMatchResult::NoMatch {
                            diagnostic_seed: TypeRelationDiagnosticSeed::Nominal { error: NominalModuleError::UnknownType { .. } },
                        } => {
                            if let Some(result) = self.match_named_trait_parameter(&actual_name, expected_name) {
                                return result;
                            }
                        }
                        ParameterMatchResult::NoMatch { diagnostic_seed: TypeRelationDiagnosticSeed::Nominal { .. } } => return nominal,
                        ParameterMatchResult::NoMatch { .. } => {
                            if let Some(result) = self.match_named_trait_parameter(&actual_name, expected_name) {
                                return result;
                            }
                            return nominal;
                        }
                        ParameterMatchResult::Trait { .. } | ParameterMatchResult::Row => {
                            unreachable!("nominal matching should not yield trait/row")
                        }
                    }
                }
            }
            ValkyrieType::TraitObject(TraitObject { trait_path, .. }) => {
                if let Some(actual_name) = named_type_name(actual) {
                    return self.match_named_trait_parameter(&actual_name, trait_path).unwrap_or(ParameterMatchResult::NoMatch {
                        diagnostic_seed: TypeRelationDiagnosticSeed::IncompatibleTypes { actual: actual.clone(), expected: expected.clone() },
                    });
                }
            }
            ValkyrieType::Row(row) => {
                if let Some(actual_name) = named_type_name(actual) {
                    return self.match_row_requirement(&actual_name, &row_type_to_requirement(row));
                }
            }
            _ => {}
        }

        ParameterMatchResult::NoMatch {
            diagnostic_seed: TypeRelationDiagnosticSeed::IncompatibleTypes { actual: actual.clone(), expected: expected.clone() },
        }
    }

    pub fn match_nominal_parameter(&self, actual_name: &Identifier, expected_name: &Identifier) -> ParameterMatchResult {
        match self.nominal.nominal_match_distance(actual_name, expected_name) {
            Ok(Some(0)) => ParameterMatchResult::NominalExact,
            Ok(Some(distance)) => ParameterMatchResult::NominalSubtype { distance },
            Ok(None) => ParameterMatchResult::NoMatch {
                diagnostic_seed: TypeRelationDiagnosticSeed::IncompatibleTypes {
                    actual: ValkyrieType::Named(actual_name.clone()),
                    expected: ValkyrieType::Named(expected_name.clone()),
                },
            },
            Err(error) => ParameterMatchResult::NoMatch { diagnostic_seed: TypeRelationDiagnosticSeed::Nominal { error } },
        }
    }

    pub fn match_named_trait_parameter(&self, actual_name: &Identifier, trait_name: &Identifier) -> Option<ParameterMatchResult> {
        match self.traits.satisfy_named_trait(actual_name, trait_name) {
            Ok(witness) => Some(ParameterMatchResult::Trait { witness }),
            Err(TraitModuleError::UnknownTrait { .. }) => None,
            Err(error) => Some(ParameterMatchResult::NoMatch { diagnostic_seed: TypeRelationDiagnosticSeed::Trait { error } }),
        }
    }

    pub fn match_row_requirement(&self, actual_name: &Identifier, requirement: &RowRequirement) -> ParameterMatchResult {
        let Some(candidate) = self.structs.get(actual_name)
        else {
            return ParameterMatchResult::NoMatch {
                diagnostic_seed: TypeRelationDiagnosticSeed::UnsupportedActualType { actual: ValkyrieType::Named(actual_name.clone()) },
            };
        };

        match requirement.check_struct(candidate) {
            Ok(()) => ParameterMatchResult::Row,
            Err(errors) => ParameterMatchResult::NoMatch { diagnostic_seed: TypeRelationDiagnosticSeed::Row { errors } },
        }
    }
}

fn row_type_to_requirement(row: &RowType) -> RowRequirement {
    RowRequirement::from_methods(
        row.methods
            .iter()
            .map(|method| RowMethodSignature {
                name: method.name.clone(),
                params: method.params.clone(),
                return_type: method.return_type.clone(),
            })
            .collect(),
    )
}

fn named_type_name(ty: &ValkyrieType) -> Option<Identifier> {
    match ty {
        ValkyrieType::Named(name) => Some(name.clone()),
        ValkyrieType::Apply(base, _) => named_type_name(base),
        _ => None,
    }
}

fn trivially_compatible(actual: &ValkyrieType, expected: &ValkyrieType) -> bool {
    match (actual, expected) {
        (_, ValkyrieType::r#SelfType) => true,
        (ValkyrieType::Array(actual), ValkyrieType::Array(expected)) => trivially_compatible(actual, expected),
        (ValkyrieType::Tuple(actual), ValkyrieType::Tuple(expected)) if actual.len() == expected.len() => {
            actual.iter().zip(expected).all(|(actual, expected)| trivially_compatible(actual, expected))
        }
        _ => actual == expected || matches!(actual, ValkyrieType::AutoType) || matches!(expected, ValkyrieType::AutoType),
    }
}
