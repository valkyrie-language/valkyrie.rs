#![doc = include_str!("readme.md")]

mod associated_types;
mod backend_boundary;
mod diagnostics;
mod nominal;
mod overload;
mod row;
mod support;
mod trait_system;

#[test]
fn spec_suite_is_wired() {
    assert!(true, "semantic spec suite should stay wired into tests/main.rs");
}

#[test]
fn spec_support_models_row_rules_as_implemented() {
    let case =
        support::SpecCase::implemented("row_rejects_associated_types", support::SpecLayer::Row, "anonymous rows do not carry associated types");

    assert_eq!(case.status, support::SpecStatus::Implemented);
}
