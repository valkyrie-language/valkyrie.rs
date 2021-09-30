//! Minimal backend-boundary checks.

#![allow(missing_docs)]

use crate::MirDispatchKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendRoute {
    StaticOnly,
    WitnessCapable,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BackendInputShape {
    pub contains_open_row_evidence: bool,
    pub contains_unresolved_nominal_checks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendBoundaryError {
    OpenRowEvidence,
    UnresolvedNominalCheck,
    UnsupportedTraitDispatch { route: BackendRoute },
    UnsupportedEffectDispatch { route: BackendRoute },
}

pub fn validate_backend_input(shape: BackendInputShape) -> Result<(), BackendBoundaryError> {
    if shape.contains_open_row_evidence {
        return Err(BackendBoundaryError::OpenRowEvidence);
    }

    if shape.contains_unresolved_nominal_checks {
        return Err(BackendBoundaryError::UnresolvedNominalCheck);
    }

    Ok(())
}

pub fn validate_dispatch_for_route(route: BackendRoute, dispatch: MirDispatchKind) -> Result<(), BackendBoundaryError> {
    match (route, dispatch) {
        (_, MirDispatchKind::Static) => Ok(()),
        (BackendRoute::WitnessCapable | BackendRoute::Full, MirDispatchKind::Witness) => Ok(()),
        (BackendRoute::Full, MirDispatchKind::EffectHandler) => Ok(()),
        (_, MirDispatchKind::Witness) => Err(BackendBoundaryError::UnsupportedTraitDispatch { route }),
        (_, MirDispatchKind::EffectHandler) => Err(BackendBoundaryError::UnsupportedEffectDispatch { route }),
    }
}
