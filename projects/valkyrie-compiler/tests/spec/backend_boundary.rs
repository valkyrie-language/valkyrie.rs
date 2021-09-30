use valkyrie_compiler::{
    backend_boundary::{validate_backend_input, validate_dispatch_for_route, BackendBoundaryError, BackendInputShape, BackendRoute},
    MirDispatchKind,
};

#[test]
fn backend_rejects_open_row_evidence() {
    let error =
        validate_backend_input(BackendInputShape { contains_open_row_evidence: true, contains_unresolved_nominal_checks: false }).unwrap_err();

    assert_eq!(error, BackendBoundaryError::OpenRowEvidence);
}

#[test]
fn backend_rejects_unresolved_nominal_checks() {
    let error =
        validate_backend_input(BackendInputShape { contains_open_row_evidence: false, contains_unresolved_nominal_checks: true }).unwrap_err();

    assert_eq!(error, BackendBoundaryError::UnresolvedNominalCheck);
}

#[test]
fn backend_rejects_open_trait_dispatch_when_route_cannot_lower_it() {
    let error = validate_dispatch_for_route(BackendRoute::StaticOnly, MirDispatchKind::Witness).unwrap_err();

    assert_eq!(error, BackendBoundaryError::UnsupportedTraitDispatch { route: BackendRoute::StaticOnly });
}

#[test]
fn backend_rejects_open_effect_dispatch_when_route_cannot_lower_it() {
    let error = validate_dispatch_for_route(BackendRoute::WitnessCapable, MirDispatchKind::EffectHandler).unwrap_err();

    assert_eq!(error, BackendBoundaryError::UnsupportedEffectDispatch { route: BackendRoute::WitnessCapable });
}
