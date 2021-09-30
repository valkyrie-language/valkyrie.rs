use valkyrie_compiler::hir::overload::{resolve_overload, OverloadCandidate, OverloadMatchKind, OverloadResolutionError};
use valkyrie_types::Identifier;

#[test]
fn overload_prefers_nominal_then_trait_then_row() {
    let exact = resolve_overload(&[
        OverloadCandidate::new("take_animal_exact", OverloadMatchKind::NominalExact),
        OverloadCandidate::new("take_animal_sub", OverloadMatchKind::NominalSubtype { distance: 1 }),
        OverloadCandidate::new("take_reader_trait", OverloadMatchKind::Trait),
        OverloadCandidate::new("take_row", OverloadMatchKind::Row),
    ])
    .unwrap();

    let subtype = resolve_overload(&[
        OverloadCandidate::new("take_animal_sub", OverloadMatchKind::NominalSubtype { distance: 1 }),
        OverloadCandidate::new("take_reader_trait", OverloadMatchKind::Trait),
        OverloadCandidate::new("take_row", OverloadMatchKind::Row),
    ])
    .unwrap();

    let trait_match = resolve_overload(&[
        OverloadCandidate::new("take_reader_trait", OverloadMatchKind::Trait),
        OverloadCandidate::new("take_row", OverloadMatchKind::Row),
    ])
    .unwrap();

    assert_eq!(exact.name, Identifier::new("take_animal_exact"));
    assert_eq!(subtype.name, Identifier::new("take_animal_sub"));
    assert_eq!(trait_match.name, Identifier::new("take_reader_trait"));
}

#[test]
fn row_only_overloads_are_ambiguous() {
    let error = resolve_overload(&[
        OverloadCandidate::new("take_row_a", OverloadMatchKind::Row),
        OverloadCandidate::new("take_row_b", OverloadMatchKind::Row),
    ])
    .unwrap_err();

    assert_eq!(error, OverloadResolutionError::Ambiguous { candidates: vec![Identifier::new("take_row_a"), Identifier::new("take_row_b")] });
}

#[test]
fn same_rank_trait_overloads_are_ambiguous() {
    let error = resolve_overload(&[
        OverloadCandidate::new("take_readable", OverloadMatchKind::Trait),
        OverloadCandidate::new("take_seekable", OverloadMatchKind::Trait),
    ])
    .unwrap_err();

    assert_eq!(
        error,
        OverloadResolutionError::Ambiguous { candidates: vec![Identifier::new("take_readable"), Identifier::new("take_seekable")] }
    );
}

#[test]
fn same_distance_nominal_subtypes_are_ambiguous() {
    let error = resolve_overload(&[
        OverloadCandidate::new("take_mammal", OverloadMatchKind::NominalSubtype { distance: 1 }),
        OverloadCandidate::new("take_pet", OverloadMatchKind::NominalSubtype { distance: 1 }),
    ])
    .unwrap_err();

    assert_eq!(error, OverloadResolutionError::Ambiguous { candidates: vec![Identifier::new("take_mammal"), Identifier::new("take_pet")] });
}
