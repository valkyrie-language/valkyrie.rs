//! Minimal overload ranking helpers.
//!
//! These helpers model only the settled precedence:
//! nominal exact > nominal subtype > trait > row.

#![allow(missing_docs)]

use valkyrie_types::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidate {
    pub name: Identifier,
    pub match_kind: OverloadMatchKind,
}

impl OverloadCandidate {
    pub fn new(name: &str, match_kind: OverloadMatchKind) -> Self {
        Self { name: Identifier::new(name), match_kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadMatchKind {
    NominalExact,
    NominalSubtype { distance: usize },
    Trait,
    Row,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadResolutionError {
    NoMatch,
    Ambiguous { candidates: Vec<Identifier> },
}

pub fn resolve_overload(candidates: &[OverloadCandidate]) -> Result<OverloadCandidate, OverloadResolutionError> {
    let mut ranked = candidates.iter().map(|candidate| (rank(&candidate.match_kind), candidate)).collect::<Vec<_>>();

    ranked.sort_by_key(|(rank, _)| *rank);

    let Some((best_rank, best_candidate)) = ranked.first()
    else {
        return Err(OverloadResolutionError::NoMatch);
    };

    let tied = ranked.iter().filter(|(rank, _)| rank == best_rank).map(|(_, candidate)| candidate.name.clone()).collect::<Vec<_>>();

    if tied.len() > 1 {
        Err(OverloadResolutionError::Ambiguous { candidates: tied })
    }
    else {
        Ok((*best_candidate).clone())
    }
}

fn rank(match_kind: &OverloadMatchKind) -> (u8, usize) {
    match match_kind {
        OverloadMatchKind::NominalExact => (0, 0),
        OverloadMatchKind::NominalSubtype { distance } => (1, *distance),
        OverloadMatchKind::Trait => (2, 0),
        OverloadMatchKind::Row => (3, 0),
    }
}
