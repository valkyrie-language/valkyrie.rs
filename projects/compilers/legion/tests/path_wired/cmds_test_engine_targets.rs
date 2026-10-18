//! Migrated from `src/cmds/test_engine/targets.rs` (`tests`).

use super::*;

#[test]
fn defaults_to_nyar() {
    assert_eq!(resolve_test_targets(None), vec!["legion"]);
}

#[test]
fn expands_all() {
    assert_eq!(resolve_test_targets(Some("all")), vec!["legion", "clr", "jvm", "node"]);
}

#[test]
fn splits_list() {
    assert_eq!(resolve_test_targets(Some("legion,clr")), vec!["legion", "clr"]);
    assert_eq!(resolve_test_targets(Some("nyar,node")), vec!["nyar", "node"]);
}

#[test]
fn legion_alias_parses_to_nyar_target() {
    let target = parse_target_label("legion").expect("legion");
    assert_eq!(target.to_string(), "nyar-unknown-unknown-managed");
}

#[test]
fn nyar_vm_alias_parses_to_nyar_target() {
    let target = parse_target_label("nyar-vm").expect("nyar-vm");
    assert_eq!(target.to_string(), "nyar-unknown-unknown-managed");
}
